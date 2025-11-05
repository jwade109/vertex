use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub color: Srgba,
}

impl Triangle {
    pub fn new(a: usize, b: usize, c: usize, color: Srgba) -> Self {
        Self { color }
    }
}
