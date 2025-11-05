use bevy::prelude::*;

#[derive(Resource)]
pub struct Settings {
    pub ref_image_alpha: f32,
    pub triangle_alpha: f32,
    pub blend_scale: f32,
    pub n_colors: u16,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            ref_image_alpha: 0.4,
            triangle_alpha: 1.0,
            blend_scale: 0.5,
            n_colors: 8,
        }
    }
}
