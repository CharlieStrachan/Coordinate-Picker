pub struct Grid {
    size: f32,
    visible: bool,
    snapping: bool,
}

impl Grid {
    pub fn new(size: f32, visible: bool) -> Self {
        Self {
            size,
            visible,
            snapping: false,
        }
    }

    pub fn get_size(&self) -> f32 {
        self.size
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_snapping_enabled(&self) -> bool {
        self.snapping
    }

    pub fn set_snapping(&mut self, snapping: bool) {
        self.snapping = snapping;
    }
}
