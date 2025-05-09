
use egui::{Pos2, Vec2, Rect};

pub struct Canvas {
    width: f32,
    height: f32,
    offset: Vec2,
    zoom: f32,
}

impl Canvas {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            offset: Vec2::ZERO,
            zoom: 0.5, // Start at 50% zoom
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn get_size(&self) -> (f32, f32) {
        (self.width, self.height)
    }
    
    pub fn get_width(&self) -> f32 {
        self.width
    }
    
    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn pan(&mut self, delta: Vec2) {
        self.offset += delta;
    }

    pub fn zoom_at(&mut self, factor: f32, pos: Pos2, view_rect: Rect) {
        let old_zoom = self.zoom;
        self.zoom = (self.zoom * factor).clamp(0.1, 10.0);
        
        let view_center = view_rect.center();
        let mouse_offset = pos - view_center;
        self.offset -= mouse_offset * (self.zoom / old_zoom - 1.0);
    }

    pub fn reset_view(&mut self) {
        self.offset = Vec2::ZERO;
        self.zoom = 0.5;
    }

    pub fn get_offset(&self) -> Vec2 {
        self.offset
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom
    }

    pub fn get_screen_rect(&self, view_rect: Rect) -> Rect {
        let center = view_rect.center() + self.offset;
        let half_size = Vec2::new(self.width, self.height) * 0.5 * self.zoom;
        Rect::from_center_size(center, half_size * 2.0)
    }

    pub fn screen_to_canvas_pos(&self, screen_pos: Pos2, view_rect: Rect) -> Pos2 {
        let screen_rect = self.get_screen_rect(view_rect);
        let normalized_pos = (screen_pos - screen_rect.min) / self.zoom;
        Pos2::new(normalized_pos.x, normalized_pos.y)
    }

    pub fn canvas_to_screen_pos(&self, canvas_pos: Pos2, view_rect: Rect) -> Pos2 {
        let screen_rect = self.get_screen_rect(view_rect);
        screen_rect.min + canvas_pos.to_vec2() * self.zoom
    }
}
