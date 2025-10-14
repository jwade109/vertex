use crate::drawing::*;
use crate::lpf::Lpf;
use crate::math::Vec2;
use crate::ui_element::*;

pub struct Window {
    pos: Vec2,
    dims: Vec2,
    mouse_delta: Option<Vec2>,
    is_hover: bool,
    animation: Lpf,
}

impl Window {
    pub fn new(pos: Vec2) -> Self {
        Window {
            pos,
            dims: Vec2::new(300.0, 200.0),
            mouse_delta: None,
            is_hover: false,
            animation: Lpf::new(0.0, 0.0, 0.2),
        }
    }
}

impl UiElement for Window {
    fn contains(&self, p: Vec2) -> bool {
        let p = p - self.pos;
        0.0 <= p.x && p.x <= self.dims.x && 0.0 <= p.y && p.y <= self.dims.y
    }

    fn step(&mut self) {
        self.animation.target = self.is_clicked() as u8 as f32;
        self.animation.step()
    }

    fn set_cursor_position(&mut self, t: &mut UiInput<Vec2>) {
        if let Some(p) = t.peek() {
            let p = *p;
            self.is_hover = self.contains(p);
            if self.is_hover {
                t.take();
            }
            if let Some(q) = self.mouse_delta {
                self.pos = p - q;
            }
        } else {
            self.is_hover = false;
        }
    }

    fn on_left_click_down(&mut self, t: &mut UiInput<Vec2>) {
        if let Some(p) = t.peek() {
            if self.is_hover {
                self.mouse_delta = Some(*p - self.pos);
                t.take();
            }
        } else {
            self.mouse_delta = None;
        }
    }

    fn on_left_click_release(&mut self, _t: &mut UiInput<()>) {
        self.mouse_delta = None;
    }

    fn is_hovered(&self) -> bool {
        self.is_hover
    }

    fn is_clicked(&self) -> bool {
        self.mouse_delta.is_some()
    }

    fn draw(&self, painter: &mut ShapePainter, text: &mut TextPainter) {
        draw_rect(
            painter,
            self.pos,
            self.dims,
            0.1,
            if self.is_hover { TEAL } else { GRAY },
        );

        draw_rect(
            painter,
            self.pos,
            self.dims * self.animation.actual,
            0.11,
            BLUE,
        );
    }
}
