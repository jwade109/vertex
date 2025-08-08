use crate::lpf::*;
use bevy::color::Srgba;

#[derive(Clone)]
pub struct Triangle {
    pub color: Srgba,
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub animation: Lpf,
    pub is_visible: bool,
}

impl Triangle {
    pub fn new(a: usize, b: usize, c: usize, color: Srgba) -> Self {
        Self {
            color,
            a,
            b,
            c,
            animation: Lpf::new(0.0, 0.0, 0.1),
            is_visible: false,
        }
    }
}
