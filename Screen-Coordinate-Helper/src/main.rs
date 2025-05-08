mod app;
mod canvas;
mod coordinate;
mod grid;
mod marker;
mod ui;

use app::CoordinatePickerApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1280.0, 800.0)),
        min_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    
    eframe::run_native(
        "Coordinate Picker",
        native_options,
        Box::new(|cc| Box::new(CoordinatePickerApp::new(cc)))
    )
}
