use crate::secret_project::*;

pub struct Vertex {
    pub pos: Vec2,
    pub visible_count: usize,
    pub invisible_count: usize,
}

impl Vertex {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            visible_count: 0,
            invisible_count: 0,
        }
    }
}
