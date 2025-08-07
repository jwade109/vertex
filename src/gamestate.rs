use crate::puzzle::Puzzle;
use bevy::prelude::*;

#[derive(Resource)]
pub struct GameState {
    pub puzzle: Puzzle,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            puzzle: Puzzle::new(),
        }
    }
}
