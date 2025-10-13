use crate::color_picker::ColorPicker;
use crate::puzzle::Puzzle;
use crate::take_once::*;
use bevy::prelude::*;

#[derive(Resource)]
pub struct VertexApp {
    pub mouse_pos: Option<Vec2>,
    pub puzzle: Puzzle,
    pub color_picker: ColorPicker,
    pub draw_hidden_edges: bool,
    pub ref_image_alpha: f32,
    pub triangle_alpha: f32,
}

impl VertexApp {
    pub fn new() -> Self {
        Self {
            mouse_pos: None,
            puzzle: Puzzle::new(),
            color_picker: ColorPicker::new(),
            draw_hidden_edges: true,
            ref_image_alpha: 0.4,
            triangle_alpha: 1.0,
        }
    }

    pub fn step(&mut self) {
        self.puzzle.step();
        self.color_picker.step();
    }

    pub fn set_cursor_position(&mut self, mut p: TakeOnce<Vec2>) {
        self.color_picker.set_cursor_position(&mut p);
        self.puzzle.set_cursor_position(&mut p);
    }

    pub fn on_right_mouse_release(&mut self) {
        self.color_picker.close();
        if let Some(c) = self.color_picker.selected_color() {
            self.puzzle.set_color(self.color_picker.center(), c);
        }
    }

    pub fn on_right_mouse_press(&mut self) {
        let mut t = TakeOnce::from_option(self.mouse_pos);
        self.color_picker.on_right_click_down(&mut t);
        self.puzzle.on_right_click_down(&mut t);
    }

    pub fn on_left_mouse_press(&mut self) {
        if let Some(p) = self.mouse_pos {
            self.color_picker.on_left_click_down();
            self.puzzle.on_left_click_down(p);
        }
    }

    pub fn on_left_mouse_release(&mut self) {
        self.color_picker.on_left_click_up();
        self.puzzle.on_left_click_up();
    }
}
