use crate::drawing::*;
use crate::lpf::Lpf;
use crate::math::*;
use crate::ui_element::*;
use bevy::color::Srgba;
use bevy_vector_shapes::prelude::*;

pub struct Button {
    pub text: String,
    pub pos: Vec2,
    pub dims: Vec2,
    pub hover_animation: Lpf,
    pub clicked_animation: Lpf,
    pub color: Srgba,
    pub is_hover: bool,
    pub is_clicked: bool,
}

impl Button {
    pub fn new(text: impl Into<String>, pos: Vec2, dims: Vec2) -> Self {
        Button {
            text: text.into(),
            pos,
            dims,
            hover_animation: Lpf::new(0.0, 0.0, 0.2),
            clicked_animation: Lpf::new(0.0, 0.0, 0.2),
            color: Srgba::new(rand(), rand(), rand(), 1.0),
            is_hover: false,
            is_clicked: false,
        }
    }
}

impl UiElement for Button {
    fn contains(&self, p: Vec2) -> bool {
        let p = p - self.pos;
        0.0 <= p.x && p.x <= self.dims.x && 0.0 <= p.y && p.y <= self.dims.y
    }

    fn step(&mut self) {
        self.hover_animation.target = self.is_hover as u8 as f32;
        self.hover_animation.step();
        self.clicked_animation.target = self.is_clicked() as u8 as f32;
        self.clicked_animation.step();
    }

    fn set_cursor_position(&mut self, t: &mut UiInput<Vec2>) {
        let p = t.peek();
        if let Some(p) = p {
            self.is_hover = self.contains(*p);
            if self.is_hover {
                t.take();
            }
        } else {
            self.is_hover = false;
        }
    }

    fn is_hovered(&self) -> bool {
        self.is_hover
    }

    fn on_left_click_down(&mut self, t: &mut UiInput<Vec2>) {
        if self.is_hover {
            if let Some(p) = t.peek() {
                self.is_clicked = self.contains(*p);
                if self.is_clicked {
                    t.take();
                }
            }
        } else {
            self.is_clicked = false;
        }
    }

    fn on_left_click_release(&mut self, t: &mut UiInput<()>) {
        if let Some(_) = t.peek() {
            if self.is_clicked {}
            self.is_clicked = false;
        }
    }

    fn is_clicked(&self) -> bool {
        self.is_clicked
    }

    fn draw(&self, painter: &mut ShapePainter, text: &mut TextPainter) {
        painter.reset();

        let offset = Vec2::splat(5.0);

        let pos = self.pos - if self.is_clicked { offset } else { Vec2::ZERO };

        let c1 = WHITE;

        painter.set_color(c1);
        painter.set_translation((pos + self.dims / 2.0).extend(0.28));
        painter.rect(self.dims);

        // drop shadow
        draw_rect(
            painter,
            self.pos - Vec2::splat(5.0),
            self.dims,
            0.1,
            LIGHT_GRAY,
        );

        let c2 = self.color.with_alpha(0.7);
        let anim_dims = self
            .dims
            .with_x((self.dims.x * self.hover_animation.actual).max(7.0));
        painter.set_color(c2);
        painter.set_translation((pos + anim_dims / 2.0).extend(0.29));
        painter.rect(anim_dims);

        if self.clicked_animation.actual > 0.01 {
            let c2 = self.color;
            let anim_dims = self
                .dims
                .with_x(self.dims.x * self.clicked_animation.actual);
            painter.set_color(c2);
            painter.set_translation((pos + anim_dims / 2.0).extend(0.29));
            painter.rect(anim_dims);
        }

        text.set_position((pos + self.dims / 2.0).extend(100.0));
        text.set_height(self.dims.y * 0.7);
        text.set_color(BLACK.with_alpha(1.0));
        text.text(self.text.clone());
    }
}
