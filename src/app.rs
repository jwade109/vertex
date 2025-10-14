use bevy::prelude::*;

#[derive(Resource)]
pub struct VertexApp {
    pub mouse_pos: Option<Vec2>,
    pub draw_hidden_edges: bool,
    pub ref_image_alpha: f32,
    pub triangle_alpha: f32,
    pub puzzle_locked: bool,
}

impl VertexApp {
    pub fn new() -> Self {
        Self {
            mouse_pos: None,
            draw_hidden_edges: true,
            ref_image_alpha: 0.4,
            triangle_alpha: 1.0,
            puzzle_locked: false,
        }
    }
}
