use crate::puzzle::Puzzle;
use bevy::prelude::*;

#[derive(Resource)]
pub struct GameState {
    pub mouse_pos: Option<Vec2>,
    pub puzzle: Puzzle,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            mouse_pos: None,
            puzzle: Puzzle::new(),
        }
    }
}
