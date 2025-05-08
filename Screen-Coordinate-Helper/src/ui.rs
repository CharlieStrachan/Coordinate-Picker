use egui::{Color32, Pos2};

#[derive(Default)]
pub struct UiState {
    // Canvas/resolution settings
    pub selected_resolution: String,
    pub custom_width: f32,
    pub custom_height: f32,
    
    // Grid settings
    pub show_grid: bool,
    pub grid_size: f32,
    pub enable_snapping: bool,
    
    // Coordinate system settings
    pub origin_top_left: bool,
    
    // Marker settings
    pub marker_color: Color32,
    
    // Current position tracking
    pub current_position: Pos2,
    pub current_position_raw: Pos2,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selected_resolution: "Custom".to_string(),
            custom_width: 800.0,
            custom_height: 600.0,
            show_grid: true,
            grid_size: 20.0,
            enable_snapping: true,
            origin_top_left: true,
            marker_color: Color32::from_rgb(0, 120, 255),
            current_position: Pos2::ZERO,
            current_position_raw: Pos2::ZERO,
        }
    }
}
