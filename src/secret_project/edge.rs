#![allow(unused)]

use crate::*;

pub struct Edge {
    pub a: usize,
    pub b: usize,
    pub length_animation: Lpf,
    pub thickness_animation: Lpf,
}

impl Edge {
    pub fn new(a: usize, b: usize, active: bool) -> Self {
        let animation = active as u8 as f32;
        Self {
            a,
            b,
            length_animation: Lpf::new(animation, animation, 0.1),
            thickness_animation: Lpf::new(3.0, 1.0, 0.1),
        }
    }

    pub fn has_vertex(&self, id: usize) -> bool {
        self.a == id || self.b == id
    }
}
