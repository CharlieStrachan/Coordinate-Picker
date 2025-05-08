use crate::canvas::Canvas;
use crate::coordinate::CoordinateSystem;
use crate::grid::Grid;
use crate::marker::Marker;
use crate::ui::UiState;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use egui::{Color32, Context, Stroke, Ui};
use std::collections::HashMap;

pub struct CoordinatePickerApp {
    canvas: Canvas,
    grid: Grid,
    coordinate_system: CoordinateSystem,
    markers: Vec<Marker>,
    ui_state: UiState,
    clipboard: Option<ClipboardContext>,
    resolution_presets: HashMap<String, (f32, f32)>,
}

impl CoordinatePickerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Set up custom fonts and styles if needed
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        cc.egui_ctx.set_style(style);

        // Initialize clipboard
        let clipboard = ClipboardProvider::new().ok();

        // Define common resolution presets
        let mut resolution_presets = HashMap::new();
        resolution_presets.insert("HD (1280x720)".to_string(), (1280.0, 720.0));
        resolution_presets.insert("Full HD (1920x1080)".to_string(), (1920.0, 1080.0));
        resolution_presets.insert("4K (3840x2160)".to_string(), (3840.0, 2160.0));
        resolution_presets.insert("iPhone (390x844)".to_string(), (390.0, 844.0));
        resolution_presets.insert("iPad (810x1080)".to_string(), (810.0, 1080.0));
        resolution_presets.insert("Custom".to_string(), (800.0, 600.0));

        // Create initial instance
        let mut app = Self {
            canvas: Canvas::new(1920.0, 1080.0), // Start with Full HD as default
            grid: Grid::new(45.0, true), // Grid size of 45px works better for a 1920x1080 canvas
            coordinate_system: CoordinateSystem::new(true), // Default to top-left origin
            markers: Vec::new(),
            ui_state: UiState::default(),
            clipboard,
            resolution_presets,
        };
        
        // Apply all default settings immediately
        app.grid.set_size(app.ui_state.grid_size);
        app.grid.set_visible(app.ui_state.show_grid);
        app.grid.set_snapping(app.ui_state.enable_snapping);
        app.coordinate_system.set_origin_top_left(app.ui_state.origin_top_left);
        
        // The UI state has 1920x1080 as default in UiState::default()
        app.update_canvas_resolution();
        
        app
    }

    pub fn copy_to_clipboard(&mut self, text: String) -> bool {
        if let Some(clipboard) = &mut self.clipboard {
            clipboard.set_contents(text).is_ok()
        } else {
            false
        }
    }

    fn update_canvas_resolution(&mut self) {
        if let Some((width, height)) = self.resolution_presets.get(&self.ui_state.selected_resolution) {
            if self.ui_state.selected_resolution == "Custom" {
                self.canvas.set_size(self.ui_state.custom_width, self.ui_state.custom_height);
            } else {
                self.canvas.set_size(*width, *height);
                self.ui_state.custom_width = *width;
                self.ui_state.custom_height = *height;
            }
        }
    }

    fn apply_grid_snapping(&self, pos: egui::Pos2) -> egui::Pos2 {
        if self.grid.is_snapping_enabled() {
            let grid_size = self.grid.get_size();
            let (canvas_width, canvas_height) = self.canvas.get_size();
            
            // Ensure we snap to exact grid lines starting from (0,0)
            // This is crucial for making visual grid and snap points align
            let x = (pos.x / grid_size).round() * grid_size;
            let y = (pos.y / grid_size).round() * grid_size;
            
            // Ensure we can select corners including (0,0) and max bounds
            if pos.x < grid_size / 2.0 {
                // If close to left edge, snap to 0
                egui::pos2(0.0, y)
            } else if pos.x > canvas_width - grid_size / 2.0 {
                // If close to right edge, snap to max width
                egui::pos2(canvas_width, y)
            } else if pos.y < grid_size / 2.0 {
                // If close to top edge, snap to 0
                egui::pos2(x, 0.0)
            } else if pos.y > canvas_height - grid_size / 2.0 {
                // If close to bottom edge, snap to max height
                egui::pos2(x, canvas_height)
            } else {
                // Otherwise snap to nearest grid intersection
                egui::pos2(x, y)
            }
        } else {
            pos
        }
    }

    fn handle_canvas_interactions(&mut self, ui: &mut Ui, response: egui::Response) {
        let canvas_rect = response.rect;

        // Handle panning with middle mouse button or Alt+Left button
        if response.dragged_by(egui::PointerButton::Middle) || 
           (response.dragged_by(egui::PointerButton::Primary) && ui.input(|i| i.modifiers.alt)) {
            self.canvas.pan(response.drag_delta());
        }

        // Handle zooming with scroll
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.scroll_delta.y);
            if scroll_delta != 0.0 {
                let zoom_factor = if scroll_delta > 0.0 { 1.1 } else { 1.0 / 1.1 };
                let mouse_pos = ui.input(|i| i.pointer.hover_pos());
                if let Some(pos) = mouse_pos {
                    self.canvas.zoom_at(zoom_factor, pos, canvas_rect);
                }
            }
        }

        // Update current mouse position
        if let Some(mouse_pos) = response.hover_pos() {
            let canvas_pos = self.canvas.screen_to_canvas_pos(mouse_pos, canvas_rect);
            let snapped_pos = if self.grid.is_snapping_enabled() {
                // Use the improved grid snapping function for consistency
                self.apply_grid_snapping(canvas_pos)
            } else {
                canvas_pos
            };

            self.ui_state.current_position = self.coordinate_system.to_system_coordinates(snapped_pos);
            self.ui_state.current_position_raw = self.coordinate_system.to_system_coordinates(canvas_pos);
        }

        // Handle clicking to add markers
        if response.clicked() {
            if let Some(pos) = response.hover_pos() {
                // Check if the click is within the canvas border
                let border_rect = self.canvas.get_screen_rect(canvas_rect);
                if border_rect.contains(pos) {
                    let canvas_pos = self.canvas.screen_to_canvas_pos(pos, canvas_rect);
                    let snapped_pos = if self.grid.is_snapping_enabled() {
                        self.apply_grid_snapping(canvas_pos)
                    } else {
                        canvas_pos
                    };
                    
                    // Additional check to ensure the final position is within canvas bounds
                    // Get canvas dimensions from get_size() instead of individual methods
                    let (canvas_width, canvas_height) = self.canvas.get_size();
                    
                    if snapped_pos.x >= 0.0 && snapped_pos.x <= canvas_width && 
                       snapped_pos.y >= 0.0 && snapped_pos.y <= canvas_height {
                        let system_pos = self.coordinate_system.to_system_coordinates(snapped_pos);
                        let marker = Marker::new(snapped_pos, system_pos, self.ui_state.marker_color);
                        self.markers.push(marker);
                    }
                }
            }
        }

        // Handle marker deletion with right-click
        if response.secondary_clicked() {
            if let Some(pos) = response.hover_pos() {
                // Check if the click is within the canvas border
                let border_rect = self.canvas.get_screen_rect(canvas_rect);
                if border_rect.contains(pos) {
                    let canvas_pos = self.canvas.screen_to_canvas_pos(pos, canvas_rect);
                    self.remove_nearby_marker(canvas_pos);
                }
            }
        }
    }

    fn remove_nearby_marker(&mut self, position: egui::Pos2) {
        const CLICK_THRESHOLD: f32 = 10.0;
        
        if let Some(index) = self.markers.iter().position(|marker| {
            let delta = marker.position - position;
            delta.length() < CLICK_THRESHOLD
        }) {
            self.markers.remove(index);
        }
    }

    fn draw_canvas(&self, ui: &mut Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            egui::Sense::click_and_drag()
        );

        let canvas_rect = response.rect;

        // Draw background (respecting dark/light mode)
        let bg_color = if self.ui_state.dark_mode {
            Color32::from_rgb(20, 20, 20) // Dark background
        } else {
            Color32::from_rgb(240, 240, 240) // Light background
        };
        painter.rect_filled(canvas_rect, 0.0, bg_color);

        // Calculate canvas border rect once
        let border_rect = self.canvas.get_screen_rect(canvas_rect);
        
        // Draw grid (constrained to border)
        if self.grid.is_visible() {
            self.draw_grid(&painter, canvas_rect, border_rect);
        }

        // Draw canvas border based on the configured resolution
        // Make border color adapt to dark/light mode
        let border_color = if self.ui_state.dark_mode {
            Color32::from_rgb(150, 150, 150) // Lighter in dark mode
        } else {
            Color32::from_rgb(100, 100, 100) // Dark in light mode
        };
        painter.rect_stroke(
            border_rect,
            0.0,
            Stroke::new(2.0, border_color)
        );

        // Draw markers
        for marker in &self.markers {
            let screen_pos = self.canvas.canvas_to_screen_pos(marker.position, canvas_rect);
            
            // Draw marker circle
            painter.circle_filled(
                screen_pos, 
                5.0, 
                marker.color
            );
            
            // Draw marker label
            let label_pos = screen_pos + egui::vec2(10.0, 0.0);
            // Theme-aware text color for marker labels
            let text_color = if self.ui_state.dark_mode {
                Color32::WHITE
            } else {
                Color32::BLACK
            };
            painter.text(
                label_pos,
                egui::Align2::LEFT_CENTER,
                format!("({}, {})", marker.system_position.x as i32, marker.system_position.y as i32),
                egui::FontId::default(),
                text_color,
            );
        }

        // Draw crosshair at mouse position
        if let Some(mouse_pos) = response.hover_pos() {
            let crosshair_color = Color32::from_rgb(255, 0, 0);
            let crosshair_size = 10.0;
            
            // Horizontal line
            painter.line_segment(
                [
                    egui::pos2(mouse_pos.x - crosshair_size, mouse_pos.y),
                    egui::pos2(mouse_pos.x + crosshair_size, mouse_pos.y),
                ],
                Stroke::new(1.0, crosshair_color),
            );
            
            // Vertical line
            painter.line_segment(
                [
                    egui::pos2(mouse_pos.x, mouse_pos.y - crosshair_size),
                    egui::pos2(mouse_pos.x, mouse_pos.y + crosshair_size),
                ],
                Stroke::new(1.0, crosshair_color),
            );
            
            // If grid snapping is enabled, draw a snap indicator
            if self.grid.is_snapping_enabled() {
                let canvas_pos = self.canvas.screen_to_canvas_pos(mouse_pos, canvas_rect);
                let snapped_pos = self.apply_grid_snapping(canvas_pos);
                let snapped_screen_pos = self.canvas.canvas_to_screen_pos(snapped_pos, canvas_rect);
                
                // Draw snap indicator circle
                painter.circle_stroke(
                    snapped_screen_pos,
                    8.0,
                    Stroke::new(1.5, Color32::from_rgb(0, 200, 0)),
                );
                
                // Draw a line from cursor to snap point if they're not the same
                if (snapped_screen_pos - mouse_pos).length() > 2.0 {
                    painter.line_segment(
                        [mouse_pos, snapped_screen_pos],
                        Stroke::new(1.0, Color32::from_rgba_premultiplied(0, 200, 0, 150)),
                    );
                }
            }
        }

        response
    }

    fn draw_grid(&self, painter: &egui::Painter, canvas_rect: egui::Rect, border_rect: egui::Rect) {
        let grid_size = self.grid.get_size() * self.canvas.get_zoom();
        if grid_size < 5.0 {
            return; // Grid too small to be useful
        }

        // Use darker grid color based on theme
        let grid_color = if self.ui_state.dark_mode {
            Color32::from_rgba_premultiplied(180, 180, 180, 60) // Lighter in dark mode
        } else {
            Color32::from_rgba_premultiplied(80, 80, 80, 80) // Darker in light mode
        };
        
        // Get canvas dimensions and ensure grid aligns with (0,0)
        let (canvas_width, canvas_height) = self.canvas.get_size();
        
        // Draw grid starting at (0,0) of the canvas and extending to its borders
        // This ensures grid lines and snap points align perfectly
        
        // First, calculate where (0,0) is in screen space
        let origin_screen_pos = self.canvas.canvas_to_screen_pos(egui::pos2(0.0, 0.0), canvas_rect);
        
        // Calculate how many grid cells from origin to draw in each direction
        let cells_left = (origin_screen_pos.x - border_rect.min.x) / grid_size;
        let cells_right = (border_rect.max.x - origin_screen_pos.x) / grid_size;
        let cells_up = (origin_screen_pos.y - border_rect.min.y) / grid_size;
        let cells_down = (border_rect.max.y - origin_screen_pos.y) / grid_size;
        
        let left_count = cells_left.ceil() as i32 + 2;  // Add extra cells for safety
        let right_count = cells_right.ceil() as i32 + 2;
        let up_count = cells_up.ceil() as i32 + 2;
        let down_count = cells_down.ceil() as i32 + 2;
        
        // Draw vertical grid lines - start from origin and go left/right
        for i in -left_count..=right_count {
            // Calculate exact screen position for this grid line
            let canvas_x = (i as f32) * self.grid.get_size();
            let screen_x = self.canvas.canvas_to_screen_pos(egui::pos2(canvas_x, 0.0), canvas_rect).x;
            
            // Only draw if inside the border
            if screen_x >= border_rect.min.x && screen_x <= border_rect.max.x {
                painter.line_segment(
                    [
                        egui::pos2(screen_x, border_rect.min.y),
                        egui::pos2(screen_x, border_rect.max.y),
                    ],
                    Stroke::new(1.0, grid_color),
                );
            }
        }
        
        // Draw horizontal grid lines - start from origin and go up/down
        for i in -up_count..=down_count {
            // Calculate exact screen position for this grid line
            let canvas_y = (i as f32) * self.grid.get_size();
            let screen_y = self.canvas.canvas_to_screen_pos(egui::pos2(0.0, canvas_y), canvas_rect).y;
            
            // Only draw if inside the border
            if screen_y >= border_rect.min.y && screen_y <= border_rect.max.y {
                painter.line_segment(
                    [
                        egui::pos2(border_rect.min.x, screen_y),
                        egui::pos2(border_rect.max.x, screen_y),
                    ],
                    Stroke::new(1.0, grid_color),
                );
            }
        }
        
        // Draw border grid lines with slightly stronger color
        let border_grid_color = if self.ui_state.dark_mode {
            Color32::from_rgba_premultiplied(200, 200, 200, 100) // Stronger in dark mode
        } else {
            Color32::from_rgba_premultiplied(100, 100, 100, 100) // Stronger in light mode
        };
        
        // Draw the canvas edges as stronger grid lines
        // Left edge (x = 0)
        let left_edge_x = self.canvas.canvas_to_screen_pos(egui::pos2(0.0, 0.0), canvas_rect).x;
        if left_edge_x >= border_rect.min.x && left_edge_x <= border_rect.max.x {
            painter.line_segment(
                [
                    egui::pos2(left_edge_x, border_rect.min.y),
                    egui::pos2(left_edge_x, border_rect.max.y),
                ],
                Stroke::new(1.5, border_grid_color),
            );
        }
        
        // Right edge (x = width)
        let right_edge_x = self.canvas.canvas_to_screen_pos(egui::pos2(canvas_width, 0.0), canvas_rect).x;
        if right_edge_x >= border_rect.min.x && right_edge_x <= border_rect.max.x {
            painter.line_segment(
                [
                    egui::pos2(right_edge_x, border_rect.min.y),
                    egui::pos2(right_edge_x, border_rect.max.y),
                ],
                Stroke::new(1.5, border_grid_color),
            );
        }
        
        // Top edge (y = 0)
        let top_edge_y = self.canvas.canvas_to_screen_pos(egui::pos2(0.0, 0.0), canvas_rect).y;
        if top_edge_y >= border_rect.min.y && top_edge_y <= border_rect.max.y {
            painter.line_segment(
                [
                    egui::pos2(border_rect.min.x, top_edge_y),
                    egui::pos2(border_rect.max.x, top_edge_y),
                ],
                Stroke::new(1.5, border_grid_color),
            );
        }
        
        // Bottom edge (y = height)
        let bottom_edge_y = self.canvas.canvas_to_screen_pos(egui::pos2(0.0, canvas_height), canvas_rect).y;
        if bottom_edge_y >= border_rect.min.y && bottom_edge_y <= border_rect.max.y {
            painter.line_segment(
                [
                    egui::pos2(border_rect.min.x, bottom_edge_y),
                    egui::pos2(border_rect.max.x, bottom_edge_y),
                ],
                Stroke::new(1.5, border_grid_color),
            );
        }
        
        // Draw origin point if visible
        let origin = self.canvas.canvas_to_screen_pos(egui::pos2(0.0, 0.0), canvas_rect);
        if canvas_rect.contains(origin) {
            painter.circle_filled(origin, 5.0, Color32::RED);
            // Use text color that works with both dark and light mode
            let text_color = if self.ui_state.dark_mode {
                Color32::WHITE
            } else {
                Color32::BLACK
            };
            painter.text(
                origin + egui::vec2(10.0, -10.0),
                egui::Align2::LEFT_BOTTOM,
                "(0, 0)",
                egui::FontId::default(),
                text_color,
            );
        }
    }
}

impl eframe::App for CoordinatePickerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Apply theme based on dark mode setting
        let mut style = (*ctx.style()).clone();
        if self.ui_state.dark_mode {
            style.visuals = egui::Visuals::dark();
        } else {
            style.visuals = egui::Visuals::light();
        }
        ctx.set_style(style);
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Coordinate Picker");
                ui.separator();
                if ui.button("Reset View").clicked() {
                    self.canvas.reset_view();
                }
                if ui.button("Clear Markers").clicked() {
                    self.markers.clear();
                }
                ui.separator();
                ui.label("Zoom:");
                let zoom_percentage = (self.canvas.get_zoom() * 100.0) as i32;
                ui.label(format!("{}%", zoom_percentage));
            });
        });

        egui::SidePanel::right("settings_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Settings");
                ui.separator();

                // Resolution settings
                ui.collapsing("Canvas Size", |ui| {
                    egui::ComboBox::from_label("Resolution")
                        .selected_text(&self.ui_state.selected_resolution)
                        .show_ui(ui, |ui| {
                            for preset in self.resolution_presets.keys() {
                                ui.selectable_value(&mut self.ui_state.selected_resolution, preset.clone(), preset);
                            }
                        });

                    if self.ui_state.selected_resolution == "Custom" {
                        ui.horizontal(|ui| {
                            ui.label("Width:");
                            ui.add(egui::DragValue::new(&mut self.ui_state.custom_width)
                                .speed(1.0)
                                .clamp_range(100.0..=10000.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Height:");
                            ui.add(egui::DragValue::new(&mut self.ui_state.custom_height)
                                .speed(1.0)
                                .clamp_range(100.0..=10000.0));
                        });
                    }

                    // Automatically apply resolution when changed
                    self.update_canvas_resolution();
                });

                // Grid settings
                ui.collapsing("Grid", |ui| {
                    // Check if grid settings have changed
                    let grid_visible_changed = ui.checkbox(&mut self.ui_state.show_grid, "Show Grid").changed();
                    
                    let mut grid_size_changed = false;
                    ui.horizontal(|ui| {
                        ui.label("Grid Size:");
                        grid_size_changed = ui.add(egui::DragValue::new(&mut self.ui_state.grid_size)
                            .speed(1.0)
                            .clamp_range(5.0..=100.0)).changed();
                    });
                    
                    let grid_snap_changed = ui.checkbox(&mut self.ui_state.enable_snapping, "Snap to Grid").changed();
                    
                    // Apply settings immediately if any value changed
                    if grid_visible_changed || grid_size_changed || grid_snap_changed {
                        self.grid.set_size(self.ui_state.grid_size);
                        self.grid.set_visible(self.ui_state.show_grid);
                        self.grid.set_snapping(self.ui_state.enable_snapping);
                    }
                });

                // Coordinate system settings
                ui.collapsing("Coordinate System", |ui| {
                    let changed1 = ui.radio_value(&mut self.ui_state.origin_top_left, true, "Origin at Top-Left (0,0)").changed();
                    let changed2 = ui.radio_value(&mut self.ui_state.origin_top_left, false, "Origin at Bottom-Left (0,0)").changed();
                    
                    // Apply immediately if changed
                    if changed1 || changed2 {
                        self.coordinate_system.set_origin_top_left(self.ui_state.origin_top_left);
                    }
                });

                // Marker settings
                ui.collapsing("Markers", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Marker Color:");
                        egui::color_picker::color_edit_button_srgba(
                            ui,
                            &mut self.ui_state.marker_color,
                            egui::color_picker::Alpha::Opaque,
                        );
                    });
                });

                ui.separator();
                
                // Current position display
                ui.heading("Current Position");
                ui.horizontal(|ui| {
                    let x = self.ui_state.current_position.x as i32;
                    let y = self.ui_state.current_position.y as i32;
                    let coords_text = format!("({}, {})", x, y);
                    ui.label(coords_text.clone());
                    if ui.button("Copy").clicked() {
                        self.copy_to_clipboard(coords_text);
                    }
                });
                
                if self.grid.is_snapping_enabled() {
                    ui.label("Snapping enabled");
                } else {
                    let x = self.ui_state.current_position_raw.x as f32;
                    let y = self.ui_state.current_position_raw.y as f32;
                    ui.label(format!("Raw: ({:.1}, {:.1})", x, y));
                }

                ui.separator();
                
                // Markers list
                ui.heading("Saved Markers");
                
                // Keep track of markers to remove 
                let mut marker_to_remove: Option<usize> = None;
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Process the markers to get coordinates ahead of time
                    let markers_data: Vec<(usize, i32, i32, String)> = self.markers.iter().enumerate()
                        .map(|(i, marker)| {
                            let x = marker.system_position.x as i32;
                            let y = marker.system_position.y as i32;
                            let coords = format!("{}, {}", x, y);
                            (i, x, y, coords)
                        })
                        .collect();
                    
                    // Now display the markers with the pre-computed data
                    for (i, x, y, coords) in markers_data {
                        let marker_text = format!("{}. ({}, {})", i + 1, x, y);
                        ui.horizontal(|ui| {
                            ui.label(marker_text);
                            
                            // Individual copy button for each marker
                            if ui.button("Copy").clicked() {
                                self.copy_to_clipboard(coords.clone());
                            }
                            
                            // Individual delete button for each marker
                            if ui.button("Delete").clicked() {
                                marker_to_remove = Some(i);
                            }
                        });
                    }
                });
                
                // Remove marker if delete button was clicked
                if let Some(index) = marker_to_remove {
                    if index < self.markers.len() {
                        self.markers.remove(index);
                    }
                }
                
                ui.separator();
                
                // Appearance settings
                ui.collapsing("Appearance", |ui| {
                    if ui.checkbox(&mut self.ui_state.dark_mode, "Dark Mode").clicked() {
                        // Theme change is applied automatically in the update method
                    }
                });
                
                // Help/Instructions
                ui.collapsing("Help", |ui| {
                    ui.label("• Click to place a marker");
                    ui.label("• Right-click to remove a marker at cursor position");
                    ui.label("• Use 'Delete' button to remove specific markers from the list");
                    ui.label("• Middle-click or Alt+drag to pan");
                    ui.label("• Scroll to zoom in/out");
                    ui.label("• Adjust grid settings for precise positioning");
                    ui.label("• Grid snapping finds the nearest grid intersection to your cursor");
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let response = self.draw_canvas(ui);
            self.handle_canvas_interactions(ui, response);
        });

        // Request continuous repainting for smooth crosshair movement
        ctx.request_repaint();
    }
}
