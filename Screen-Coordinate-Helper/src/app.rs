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

        Self {
            canvas: Canvas::new(800.0, 600.0),
            grid: Grid::new(20.0, true),
            coordinate_system: CoordinateSystem::new(true), // Default to top-left origin
            markers: Vec::new(),
            ui_state: UiState::default(),
            clipboard,
            resolution_presets,
        }
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
            let x = (pos.x / grid_size).round() * grid_size;
            let y = (pos.y / grid_size).round() * grid_size;
            egui::pos2(x, y)
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
                let grid_size = self.grid.get_size();
                let x = (canvas_pos.x / grid_size).round() * grid_size;
                let y = (canvas_pos.y / grid_size).round() * grid_size;
                egui::pos2(x, y)
            } else {
                canvas_pos
            };

            self.ui_state.current_position = self.coordinate_system.to_system_coordinates(snapped_pos);
            self.ui_state.current_position_raw = self.coordinate_system.to_system_coordinates(canvas_pos);
        }

        // Handle clicking to add markers
        if response.clicked() {
            if let Some(pos) = response.hover_pos() {
                let canvas_pos = self.canvas.screen_to_canvas_pos(pos, canvas_rect);
                let snapped_pos = if self.grid.is_snapping_enabled() {
                    self.apply_grid_snapping(canvas_pos)
                } else {
                    canvas_pos
                };
                
                let system_pos = self.coordinate_system.to_system_coordinates(snapped_pos);
                let marker = Marker::new(snapped_pos, system_pos, self.ui_state.marker_color);
                self.markers.push(marker);
            }
        }

        // Handle marker deletion with right-click
        if response.secondary_clicked() {
            if let Some(pos) = response.hover_pos() {
                let canvas_pos = self.canvas.screen_to_canvas_pos(pos, canvas_rect);
                self.remove_nearby_marker(canvas_pos);
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

        // Draw background
        painter.rect_filled(canvas_rect, 0.0, Color32::from_rgb(240, 240, 240));

        // Draw grid
        if self.grid.is_visible() {
            self.draw_grid(&painter, canvas_rect);
        }

        // Draw canvas border based on the configured resolution
        let border_rect = self.canvas.get_screen_rect(canvas_rect);
        painter.rect_stroke(
            border_rect,
            0.0,
            Stroke::new(2.0, Color32::from_rgb(100, 100, 100))
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
            painter.text(
                label_pos,
                egui::Align2::LEFT_CENTER,
                format!("({}, {})", marker.system_position.x as i32, marker.system_position.y as i32),
                egui::FontId::default(),
                Color32::BLACK,
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
        }

        response
    }

    fn draw_grid(&self, painter: &egui::Painter, canvas_rect: egui::Rect) {
        let grid_size = self.grid.get_size() * self.canvas.get_zoom();
        if grid_size < 5.0 {
            return; // Grid too small to be useful
        }

        let grid_color = Color32::from_rgba_premultiplied(100, 100, 100, 50);
        
        let offset = self.canvas.get_offset();
        let zoom = self.canvas.get_zoom();
        
        let start_x = (canvas_rect.min.x - offset.x) / grid_size;
        let start_y = (canvas_rect.min.y - offset.y) / grid_size;
        let end_x = (canvas_rect.max.x - offset.x) / grid_size;
        let end_y = (canvas_rect.max.y - offset.y) / grid_size;
        
        let start_x = start_x.floor() as i32;
        let start_y = start_y.floor() as i32;
        let end_x = end_x.ceil() as i32;
        let end_y = end_y.ceil() as i32;
        
        // Draw vertical grid lines
        for i in start_x..=end_x {
            let x = i as f32 * grid_size + offset.x;
            painter.line_segment(
                [
                    egui::pos2(x, canvas_rect.min.y),
                    egui::pos2(x, canvas_rect.max.y),
                ],
                Stroke::new(1.0, grid_color),
            );
        }
        
        // Draw horizontal grid lines
        for i in start_y..=end_y {
            let y = i as f32 * grid_size + offset.y;
            painter.line_segment(
                [
                    egui::pos2(canvas_rect.min.x, y),
                    egui::pos2(canvas_rect.max.x, y),
                ],
                Stroke::new(1.0, grid_color),
            );
        }
        
        // Draw origin point if visible
        let origin = self.canvas.canvas_to_screen_pos(egui::pos2(0.0, 0.0), canvas_rect);
        if canvas_rect.contains(origin) {
            painter.circle_filled(origin, 5.0, Color32::RED);
            painter.text(
                origin + egui::vec2(10.0, -10.0),
                egui::Align2::LEFT_BOTTOM,
                "(0, 0)",
                egui::FontId::default(),
                Color32::BLACK,
            );
        }
    }
}

impl eframe::App for CoordinatePickerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
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

                    if ui.button("Apply").clicked() {
                        self.update_canvas_resolution();
                    }
                });

                // Grid settings
                ui.collapsing("Grid", |ui| {
                    ui.checkbox(&mut self.ui_state.show_grid, "Show Grid");
                    ui.horizontal(|ui| {
                        ui.label("Grid Size:");
                        ui.add(egui::DragValue::new(&mut self.ui_state.grid_size)
                            .speed(1.0)
                            .clamp_range(5.0..=100.0));
                    });
                    ui.checkbox(&mut self.ui_state.enable_snapping, "Snap to Grid");
                    
                    if ui.button("Apply").clicked() {
                        self.grid.set_size(self.ui_state.grid_size);
                        self.grid.set_visible(self.ui_state.show_grid);
                        self.grid.set_snapping(self.ui_state.enable_snapping);
                    }
                });

                // Coordinate system settings
                ui.collapsing("Coordinate System", |ui| {
                    ui.radio_value(&mut self.ui_state.origin_top_left, true, "Origin at Top-Left (0,0)");
                    ui.radio_value(&mut self.ui_state.origin_top_left, false, "Origin at Bottom-Left (0,0)");
                    
                    if ui.button("Apply").clicked() {
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, marker) in self.markers.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let x = marker.system_position.x as i32;
                            let y = marker.system_position.y as i32;
                            let marker_text = format!("{}. ({}, {})", i + 1, x, y);
                            ui.label(marker_text.clone());
                            if ui.button("Copy").clicked() {
                                self.copy_to_clipboard(format!("{}, {}", x, y));
                            }
                        });
                    }
                });
                
                ui.separator();
                
                // Help/Instructions
                ui.collapsing("Help", |ui| {
                    ui.label("• Click to place a marker");
                    ui.label("• Right-click to remove a marker");
                    ui.label("• Middle-click or Alt+drag to pan");
                    ui.label("• Scroll to zoom in/out");
                    ui.label("• Adjust grid settings for precise positioning");
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
