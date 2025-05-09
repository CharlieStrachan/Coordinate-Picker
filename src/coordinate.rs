use egui::Pos2;

pub struct CoordinateSystem {
    origin_top_left: bool,
    canvas_height: f32,
}

impl CoordinateSystem {
    pub fn new(origin_top_left: bool) -> Self {
        Self {
            origin_top_left,
            canvas_height: 1080.0, // Default height, will be updated
        }
    }

    pub fn set_origin_top_left(&mut self, origin_top_left: bool) {
        self.origin_top_left = origin_top_left;
    }

    pub fn is_origin_top_left(&self) -> bool {
        self.origin_top_left
    }

    // Add method to update canvas height
    pub fn update_canvas_height(&mut self, height: f32) {
        self.canvas_height = height;
    }

    /// Converts canvas coordinates to the chosen coordinate system
    pub fn to_system_coordinates(&self, canvas_pos: Pos2) -> Pos2 {
        if self.origin_top_left {
            canvas_pos // Top-left origin, same as canvas
        } else {
            // Bottom-left origin, need to flip Y relative to canvas height
            Pos2::new(canvas_pos.x, self.canvas_height - canvas_pos.y)
        }
    }

    /// Converts from the chosen coordinate system back to canvas coordinates
    pub fn from_system_coordinates(&self, system_pos: Pos2) -> Pos2 {
        if self.origin_top_left {
            system_pos // Top-left origin, same as canvas
        } else {
            // Bottom-left origin, need to flip Y back relative to canvas height
            Pos2::new(system_pos.x, self.canvas_height - system_pos.y)
        }
    }
}
