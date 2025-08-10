use crate::color_picker::ColorPicker;
use crate::puzzle::Puzzle;
use bevy::prelude::*;

#[derive(Resource)]
pub struct GameState {
    pub mouse_pos: Option<Vec2>,
    pub puzzle: Puzzle,
    pub color_picker: ColorPicker,
    pub is_snapping: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            mouse_pos: None,
            puzzle: Puzzle::new(),
            color_picker: ColorPicker::new(),
            is_snapping: false,
        }
    }
}
