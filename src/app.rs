use crate::button::Button;
use crate::color_picker::ColorPicker;
use crate::drawing::*;
use crate::puzzle::Puzzle;
use crate::take_once::*;
use crate::ui_element::*;
use bevy::prelude::*;

#[derive(Resource)]
pub struct VertexApp {
    pub mouse_pos: Option<Vec2>,
    pub puzzle: Puzzle,
    pub color_picker: ColorPicker,
    pub is_snapping: bool,
    pub draw_hidden_edges: bool,
    pub ref_image_alpha: f32,
    pub triangle_alpha: f32,
    pub buttons: Vec<Box<dyn UiElement>>,
}

impl VertexApp {
    pub fn new() -> Self {
        let button_text = [
            "Load",
            "Save",
            "Picker",
            "Normalize",
            "Whatever",
            "Whocares",
        ];

        let buttons: Vec<Box<dyn UiElement>> = button_text
            .into_iter()
            .enumerate()
            .map(|(i, s)| -> Box<dyn UiElement> {
                let pos = Vec2::new(-900.0, i as f32 * 60.0);
                Box::new(Button::new(s, pos))
            })
            .collect();

        Self {
            mouse_pos: None,
            puzzle: Puzzle::new(),
            color_picker: ColorPicker::new(),
            is_snapping: false,
            draw_hidden_edges: true,
            ref_image_alpha: 0.4,
            triangle_alpha: 0.8,
            buttons,
        }
    }

    pub fn step(&mut self, commands: &mut Commands) {
        for button in &mut self.buttons {
            button.step(commands);
        }
        self.puzzle.step();
        self.color_picker.step(commands);
    }

    pub fn set_cursor_position(&mut self, mut p: TakeOnce<Vec2>) {
        for b in &mut self.buttons {
            b.set_cursor_position(&mut p);
        }
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
        let mut t = TakeOnce::from_option(self.mouse_pos);
        for elem in &mut self.buttons {
            elem.on_left_click_down(&mut t);
        }

        if let Some(p) = self.mouse_pos {
            self.color_picker.on_left_click_down();
            self.puzzle.on_left_click_down(p);
        }
    }

    pub fn on_left_mouse_release(&mut self) {
        let mut t = TakeOnce::new(());
        for elem in &mut self.buttons {
            elem.on_left_click_release(&mut t);
        }

        self.color_picker.on_left_click_up();
        self.puzzle.on_left_click_up();
    }

    pub fn draw(&self, painter: &mut ShapePainter, text: &mut TextPainter) {
        let mut y = 0.0;
        for button in self.buttons.iter().rev() {
            button.draw(painter, text);
            text.set_position(Vec3::new(0.0, y, 100.0));
            let s = format!(
                "{} {} {}",
                button.id(),
                button.is_hovered(),
                button.z_index()
            );
            text.text(s);
            y += 20.0;
        }
        self.color_picker.draw(painter, text);
    }
}
