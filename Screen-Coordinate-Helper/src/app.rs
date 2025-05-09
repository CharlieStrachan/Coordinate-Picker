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

// Main implementation of the coordinate picker app
impl CoordinatePickerApp {
    // Initialize the app with default settings
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        cc.egui_ctx.set_style(style);

        let clipboard = ClipboardProvider::new().ok();
        let mut resolution_presets = HashMap::new();
        resolution_presets.insert("HD (1280x720)".to_string(), (1280.0, 720.0));
        resolution_presets.insert("Full HD (1920x1080)".to_string(), (1920.0, 1080.0));
        resolution_presets.insert("4K (3840x2160)".to_string(), (3840.0, 2160.0));
        resolution_presets.insert("iPhone (390x844)".to_string(), (390.0, 844.0));
        resolution_presets.insert("iPad (810x1080)".to_string(), (810.0, 1080.0));
        resolution_presets.insert("Custom".to_string(), (800.0, 600.0));

        let mut app = Self {
            canvas: Canvas::new(1920.0, 1080.0),
            grid: Grid::new(45.0, true),
            coordinate_system: CoordinateSystem::new(true),
            markers: Vec::new(),
            ui_state: UiState::default(),
            clipboard,
            resolution_presets,
        };

        app.grid.set_size(app.ui_state.grid_size);
        app.grid.set_visible(app.ui_state.show_grid);
        app.grid.set_snapping(app.ui_state.enable_snapping);
        app.coordinate_system.set_origin_top_left(app.ui_state.origin_top_left);
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
                self.coordinate_system.update_canvas_height(self.ui_state.custom_height);
            } else {
                self.canvas.set_size(*width, *height);
                self.ui_state.custom_width = *width;
                self.ui_state.custom_height = *height;
                self.coordinate_system.update_canvas_height(*height);
            }
        }
    }

    // Snap cursor position to nearest grid point if enabled
    fn apply_grid_snapping(&self, pos: egui::Pos2) -> egui::Pos2 {
        if self.grid.is_snapping_enabled() {
            let grid_size = self.grid.get_size();
            let (canvas_width, canvas_height) = self.canvas.get_size();

            let x = (pos.x / grid_size).round() * grid_size;
            let y = (pos.y / grid_size).round() * grid_size;

            if pos.x < grid_size / 2.0 {
                egui::pos2(0.0, y)
            } else if pos.x > canvas_width - grid_size / 2.0 {
                egui::pos2(canvas_width, y)
            } else if pos.y < grid_size / 2.0 {
                egui::pos2(x, 0.0)
            } else if pos.y > canvas_height - grid_size / 2.0 {
                egui::pos2(x, canvas_height)
            } else {
                egui::pos2(x, y)
            }
        } else {
            pos
        }
    }

    // Handle mouse interactions with the canvas
    fn handle_canvas_interactions(&mut self, ui: &mut Ui, response: egui::Response) {
        let canvas_rect = response.rect;

        if response.dragged_by(egui::PointerButton::Middle)
            || (response.dragged_by(egui::PointerButton::Primary) && ui.input(|i| i.modifiers.alt))
        {
            self.canvas.pan(response.drag_delta());
        }

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

        if let Some(mouse_pos) = response.hover_pos() {
            let canvas_pos = self.canvas.screen_to_canvas_pos(mouse_pos, canvas_rect);
            let snapped_pos = if self.grid.is_snapping_enabled() {
                self.apply_grid_snapping(canvas_pos)
            } else {
                canvas_pos
            };

            self.ui_state.current_position = self.coordinate_system.to_system_coordinates(snapped_pos);
            self.ui_state.current_position_raw = self.coordinate_system.to_system_coordinates(canvas_pos);
        }

        if response.clicked() {
            if let Some(pos) = response.hover_pos() {
                let border_rect = self.canvas.get_screen_rect(canvas_rect);
                if border_rect.contains(pos) {
                    let canvas_pos = self.canvas.screen_to_canvas_pos(pos, canvas_rect);
                    let snapped_pos = if self.grid.is_snapping_enabled() {
                        self.apply_grid_snapping(canvas_pos)
                    } else {
                        canvas_pos
                    };

                    let (canvas_width, canvas_height) = self.canvas.get_size();

                    if snapped_pos.x >= 0.0
                        && snapped_pos.x <= canvas_width
                        && snapped_pos.y >= 0.0
                        && snapped_pos.y <= canvas_height
                    {
                        let system_pos = self.coordinate_system.to_system_coordinates(snapped_pos);
                        let marker = Marker::new(snapped_pos, system_pos, self.ui_state.marker_color);
                        self.markers.push(marker);
                    }
                }
            }
        }

        if response.secondary_clicked() {
            if let Some(pos) = response.hover_pos() {
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

    // Draw the main canvas and all its elements
    fn draw_canvas(&self, ui: &mut Ui) -> egui::Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let canvas_rect = response.rect;
        let bg_color = if self.ui_state.dark_mode {
            Color32::from_rgb(20, 20, 20)
        } else {
            Color32::from_rgb(240, 240, 240)
        };
        painter.rect_filled(canvas_rect, 0.0, bg_color);

        let border_rect = self.canvas.get_screen_rect(canvas_rect);

        if self.grid.is_visible() {
            self.draw_grid(&painter, canvas_rect, border_rect);
        }

        let border_color = if self.ui_state.dark_mode {
            Color32::from_rgb(150, 150, 150)
        } else {
            Color32::from_rgb(100, 100, 100)
        };
        painter.rect_stroke(border_rect, 0.0, Stroke::new(2.0, border_color));

        for marker in &self.markers {
            let screen_pos = self.canvas.canvas_to_screen_pos(marker.position, canvas_rect);
            painter.circle_filled(screen_pos, 5.0, marker.color);

            let label_pos = screen_pos + egui::vec2(10.0, 0.0);
            let text_color = if self.ui_state.dark_mode {
                Color32::WHITE
            } else {
                Color32::BLACK
            };
            painter.text(
                label_pos,
                egui::Align2::LEFT_CENTER,
                format!(
                    "({}, {})",
                    marker.system_position.x as i32,
                    marker.system_position.y as i32
                ),
                egui::FontId::default(),
                text_color,
            );
        }

        if let Some(mouse_pos) = response.hover_pos() {
            let crosshair_color = Color32::from_rgb(255, 0, 0);
            let crosshair_size = 10.0;

            painter.line_segment(
                [
                    egui::pos2(mouse_pos.x - crosshair_size, mouse_pos.y),
                    egui::pos2(mouse_pos.x + crosshair_size, mouse_pos.y),
                ],
                Stroke::new(1.0, crosshair_color),
            );

            painter.line_segment(
                [
                    egui::pos2(mouse_pos.x, mouse_pos.y - crosshair_size),
                    egui::pos2(mouse_pos.x, mouse_pos.y + crosshair_size),
                ],
                Stroke::new(1.0, crosshair_color),
            );

            if self.grid.is_snapping_enabled() {
                let canvas_pos = self.canvas.screen_to_canvas_pos(mouse_pos, canvas_rect);
                let snapped_pos = self.apply_grid_snapping(canvas_pos);
                let snapped_screen_pos = self.canvas.canvas_to_screen_pos(snapped_pos, canvas_rect);

                painter.circle_stroke(
                    snapped_screen_pos,
                    8.0,
                    Stroke::new(1.5, Color32::from_rgb(0, 200, 0)),
                );

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

    // Draw the grid on the canvas
    fn draw_grid(&self, painter: &egui::Painter, canvas_rect: egui::Rect, border_rect: egui::Rect) {
        let grid_size = self.grid.get_size() * self.canvas.get_zoom();
        if grid_size < 5.0 {
            return;
        }

        let grid_color = if self.ui_state.dark_mode {
            Color32::from_rgba_premultiplied(180, 180, 180, 60)
        } else {
            Color32::from_rgba_premultiplied(80, 80, 80, 80)
        };

        let (canvas_width, canvas_height) = self.canvas.get_size();
        let origin_screen_pos = self.canvas.canvas_to_screen_pos(egui::pos2(0.0, 0.0), canvas_rect);

        let cells_left = (origin_screen_pos.x - border_rect.min.x) / grid_size;
        let cells_right = (border_rect.max.x - origin_screen_pos.x) / grid_size;
        let cells_up = (origin_screen_pos.y - border_rect.min.y) / grid_size;
        let cells_down = (border_rect.max.y - origin_screen_pos.y) / grid_size;

        let left_count = cells_left.ceil() as i32 + 2;
        let right_count = cells_right.ceil() as i32 + 2;
        let up_count = cells_up.ceil() as i32 + 2;
        let down_count = cells_down.ceil() as i32 + 2;

        // Draw vertical grid lines
        for i in -left_count..=right_count {
            let canvas_x = (i as f32) * self.grid.get_size();
            let screen_x = self.canvas.canvas_to_screen_pos(egui::pos2(canvas_x, 0.0), canvas_rect).x;

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

        // Draw horizontal grid lines
        for i in -up_count..=down_count {
            let canvas_y = (i as f32) * self.grid.get_size();
            let screen_y = self.canvas.canvas_to_screen_pos(egui::pos2(0.0, canvas_y), canvas_rect).y;

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

        let border_grid_color = if self.ui_state.dark_mode {
            Color32::from_rgba_premultiplied(200, 200, 200, 100)
        } else {
            Color32::from_rgba_premultiplied(100, 100, 100, 100)
        };

        // Draw canvas edges
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

        // Draw origin point
        let origin_canvas_pos = if self.coordinate_system.is_origin_top_left() {
            egui::pos2(0.0, 0.0)
        } else {
            egui::pos2(0.0, self.canvas.get_height())
        };
        let origin = self.canvas.canvas_to_screen_pos(origin_canvas_pos, canvas_rect);
        if canvas_rect.contains(origin) {
            painter.circle_filled(origin, 5.0, Color32::RED);
            let text_color = if self.ui_state.dark_mode {
                Color32::WHITE
            } else {
                Color32::BLACK
            };
            let text_offset = if self.coordinate_system.is_origin_top_left() {
                egui::vec2(10.0, -10.0)
            } else {
                egui::vec2(10.0, 10.0)
            };
            painter.text(
                origin + text_offset,
                egui::Align2::LEFT_BOTTOM,
                "(0, 0)",
                egui::FontId::default(),
                text_color,
            );
        }
    }
}

// Implement the main update loop for the app
impl eframe::App for CoordinatePickerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Settings");
                    ui.separator();

                    ui.collapsing("Canvas Size", |ui| {
                        egui::ComboBox::from_label("Resolution")
                            .selected_text(&self.ui_state.selected_resolution)
                            .show_ui(ui, |ui| {
                                for preset in self.resolution_presets.keys() {
                                    ui.selectable_value(
                                        &mut self.ui_state.selected_resolution,
                                        preset.clone(),
                                        preset,
                                    );
                                }
                            });

                        if self.ui_state.selected_resolution == "Custom" {
                            ui.horizontal(|ui| {
                                ui.label("Width:");
                                ui.add(
                                    egui::DragValue::new(&mut self.ui_state.custom_width)
                                        .speed(1.0)
                                        .clamp_range(100.0..=10000.0),
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label("Height:");
                                ui.add(
                                    egui::DragValue::new(&mut self.ui_state.custom_height)
                                        .speed(1.0)
                                        .clamp_range(100.0..=10000.0),
                                );
                            });
                        }

                        self.update_canvas_resolution();
                    });

                    ui.collapsing("Grid", |ui| {
                        let grid_visible_changed = ui
                            .checkbox(&mut self.ui_state.show_grid, "Show Grid")
                            .changed();

                        let mut grid_size_changed = false;
                        ui.horizontal(|ui| {
                            ui.label("Grid Size:");
                            grid_size_changed = ui
                                .add(
                                    egui::DragValue::new(&mut self.ui_state.grid_size)
                                        .speed(1.0)
                                        .clamp_range(5.0..=100.0),
                                )
                                .changed();
                        });

                        let grid_snap_changed = ui
                            .checkbox(&mut self.ui_state.enable_snapping, "Snap to Grid")
                            .changed();

                        if grid_visible_changed || grid_size_changed || grid_snap_changed {
                            self.grid.set_size(self.ui_state.grid_size);
                            self.grid.set_visible(self.ui_state.show_grid);
                            self.grid.set_snapping(self.ui_state.enable_snapping);
                        }
                    });

                    ui.collapsing("Coordinate System", |ui| {
                        let changed1 = ui
                            .radio_value(
                                &mut self.ui_state.origin_top_left,
                                true,
                                "Origin at Top-Left (0,0)",
                            )
                            .changed();
                        let changed2 = ui
                            .radio_value(
                                &mut self.ui_state.origin_top_left,
                                false,
                                "Origin at Bottom-Left (0,0)",
                            )
                            .changed();
                            
                        ui.separator();
                        ui.checkbox(
                            &mut self.ui_state.recalculate_markers,
                            "Recalculate markers on origin change",
                        );

                        if changed1 || changed2 {
                            let old_origin_top_left = self.coordinate_system.is_origin_top_left();
                            self.coordinate_system
                                .set_origin_top_left(self.ui_state.origin_top_left);
                            
                            if self.ui_state.recalculate_markers && old_origin_top_left != self.ui_state.origin_top_left {
                                // Recalculate all marker positions
                                for marker in &mut self.markers {
                                    // Convert back to canvas coordinates using old system
                                    let canvas_pos = if old_origin_top_left {
                                        marker.system_position
                                    } else {
                                        egui::pos2(marker.system_position.x, self.canvas.get_height() - marker.system_position.y)
                                    };
                                    
                                    // Convert to new system coordinates
                                    marker.system_position = self.coordinate_system.to_system_coordinates(canvas_pos);
                                }
                            }
                        }
                    });

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

                    ui.heading("Saved Markers");

                    let mut marker_to_remove: Option<usize> = None;

                    if !self.markers.is_empty() {
                        if ui.button("Copy All Coordinates").clicked() {
                            let all_coords = self
                                .markers
                                .iter()
                                .enumerate()
                                .map(|(i, marker)| {
                                    let x = marker.system_position.x as i32;
                                    let y = marker.system_position.y as i32;
                                    format!("{}. ({}, {})", i + 1, x, y)
                                })
                                .collect::<Vec<String>>()
                                .join("\n");

                            self.copy_to_clipboard(all_coords);
                        }
                    }

                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            let markers_data: Vec<(usize, i32, i32, String)> = self
                                .markers
                                .iter()
                                .enumerate()
                                .map(|(i, marker)| {
                                    let x = marker.system_position.x as i32;
                                    let y = marker.system_position.y as i32;
                                    let coords = format!("{}, {}", x, y);
                                    (i, x, y, coords)
                                })
                                .collect();

                            for (i, x, y, coords) in markers_data {
                                let marker_text = format!("{}. ({}, {})", i + 1, x, y);
                                ui.horizontal(|ui| {
                                    ui.label(marker_text);

                                    if ui.button("Copy").clicked() {
                                        self.copy_to_clipboard(coords.clone());
                                    }

                                    if ui.button("Delete").clicked() {
                                        marker_to_remove = Some(i);
                                    }
                                });
                            }
                        });

                    if let Some(index) = marker_to_remove {
                        if index < self.markers.len() {
                            self.markers.remove(index);
                        }
                    }

                    ui.separator();

                    ui.collapsing("Appearance", |ui| {
                        ui.checkbox(&mut self.ui_state.dark_mode, "Dark Mode");
                    });

                    ui.collapsing("Help", |ui| {
                        ui.label("• Click to place a marker");
                        ui.label("• Right-click to remove a marker at cursor position");
                        ui.label("• Use 'Delete' button to remove specific markers from the list");
                        ui.label("• Use 'Copy All Coordinates' to copy all marker coordinates at once");
                        ui.label("• Middle-click or Alt+drag to pan");
                        ui.label("• Scroll to zoom in/out");
                        ui.label("• Adjust grid settings for precise positioning");
                        ui.label("• Grid snapping finds the nearest grid intersection to your cursor");
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let response = self.draw_canvas(ui);
            self.handle_canvas_interactions(ui, response);
        });

        ctx.request_repaint();
    }
}
