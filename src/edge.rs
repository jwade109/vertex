use crate::lpf::*;

pub struct Edge {
    pub a: usize,
    pub b: usize,
    pub portion: Lpf,
}

impl Edge {
    pub fn new(a: usize, b: usize, active: bool) -> Self {
        let portion = active as u8 as f32;
        Self {
            a,
            b,
            portion: Lpf::new(portion, portion, 0.1),
        }
    }
}
