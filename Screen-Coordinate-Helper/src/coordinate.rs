use egui::Pos2;

pub struct CoordinateSystem {
    origin_top_left: bool,
}

impl CoordinateSystem {
    pub fn new(origin_top_left: bool) -> Self {
        Self { origin_top_left }
    }

    pub fn set_origin_top_left(&mut self, origin_top_left: bool) {
        self.origin_top_left = origin_top_left;
    }

    pub fn is_origin_top_left(&self) -> bool {
        self.origin_top_left
    }

    /// Converts canvas coordinates to the chosen coordinate system
    pub fn to_system_coordinates(&self, canvas_pos: Pos2) -> Pos2 {
        if self.origin_top_left {
            canvas_pos // Top-left origin, same as canvas
        } else {
            // Bottom-left origin, need to flip Y
            Pos2::new(canvas_pos.x, -canvas_pos.y)
        }
    }

    /// Converts from the chosen coordinate system back to canvas coordinates
    pub fn from_system_coordinates(&self, system_pos: Pos2) -> Pos2 {
        if self.origin_top_left {
            system_pos // Top-left origin, same as canvas
        } else {
            // Bottom-left origin, need to flip Y back
            Pos2::new(system_pos.x, -system_pos.y)
        }
    }
}
