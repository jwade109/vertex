use crate::lpf::*;

pub struct Edge {
    pub a: usize,
    pub b: usize,
    pub portion: Lpf,
}

impl Edge {
    pub fn new(a: usize, b: usize) -> Self {
        Self {
            a,
            b,
            portion: Lpf::new(1.0, 0.0, 0.1),
        }
    }
}
