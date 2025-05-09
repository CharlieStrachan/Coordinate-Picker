use egui::{Color32, Pos2};

pub struct Marker {
    pub position: Pos2,         // Position in canvas coordinates
    pub system_position: Pos2,  // Position in the chosen coordinate system
    pub color: Color32,
}

impl Marker {
    pub fn new(position: Pos2, system_position: Pos2, color: Color32) -> Self {
        Self {
            position,
            system_position,
            color,
        }
    }
}
