use bevy::prelude::*;

#[derive(Resource)]
pub struct VertexApp {
    pub ref_image_alpha: f32,
    pub triangle_alpha: f32,
    pub puzzle_locked: bool,
    pub blend_scale: f32,
}

impl VertexApp {
    pub fn new() -> Self {
        Self {
            ref_image_alpha: 0.4,
            triangle_alpha: 1.0,
            puzzle_locked: false,
            blend_scale: 0.5,
        }
    }
}
