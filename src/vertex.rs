use crate::lpf::*;
use crate::math::*;

pub struct Vertex {
    pub pos: Vec2,
    pub marker_radius: Lpf,
    pub is_clicked: bool,
    pub is_hovered: bool,
    pub follow_count: u32,
    pub visible_count: usize,
    pub invisible_count: usize,
}

pub const MAX_FOLLOW_COUNT: u32 = 25;

impl Vertex {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            marker_radius: Lpf::new(7.0, 4.0, 0.3),
            is_clicked: false,
            is_hovered: false,
            follow_count: 0,
            visible_count: 0,
            invisible_count: 0,
        }
    }

    pub fn is_follow(&self) -> bool {
        self.follow_count >= MAX_FOLLOW_COUNT
    }
}
