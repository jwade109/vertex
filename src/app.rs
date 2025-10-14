use bevy::prelude::*;

#[derive(Resource)]
pub struct VertexApp {
    pub mouse_pos: Option<Vec2>,
    pub draw_hidden_edges: bool,
    pub ref_image_alpha: f32,
    pub triangle_alpha: f32,
    pub puzzle_locked: bool,
    pub draw_missing_edge_indicators: bool,
    pub draw_edges: bool,
    pub blend_scale: f32,
}

impl VertexApp {
    pub fn new() -> Self {
        Self {
            mouse_pos: None,
            draw_hidden_edges: true,
            ref_image_alpha: 0.4,
            triangle_alpha: 1.0,
            puzzle_locked: false,
            draw_missing_edge_indicators: false,
            draw_edges: true,
            blend_scale: 0.5,
        }
    }
}
