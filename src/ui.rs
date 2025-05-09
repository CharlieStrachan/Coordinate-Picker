
use egui::{Color32, Pos2};

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

    // Theme settings
    pub dark_mode: bool,
    pub recalculate_markers: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selected_resolution: "Full HD (1920x1080)".to_string(),
            custom_width: 1920.0,
            custom_height: 1080.0,
            show_grid: true,
            grid_size: 45.0, // Grid size of 45px works better for a 1920x1080 canvas
            enable_snapping: true,
            origin_top_left: true,
            marker_color: Color32::from_rgb(0, 120, 255),
            current_position: Pos2::ZERO,
            current_position_raw: Pos2::ZERO,
            dark_mode: true,
            recalculate_markers: true,
        }
    }
}
