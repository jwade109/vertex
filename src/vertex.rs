use crate::lpf::*;
use crate::math::*;

pub struct Vertex {
    pub pos: Vec2,
    pub marker_radius: Lpf,
    pub hidden: bool,
}

impl Vertex {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            marker_radius: Lpf::new(7.0, 4.0, 0.2),
            hidden: rand() < 0.6,
        }
    }
}
