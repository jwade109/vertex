use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub color: Srgba,
}

impl Triangle {
    pub fn new(color: Srgba) -> Self {
        Self { color }
    }
}
