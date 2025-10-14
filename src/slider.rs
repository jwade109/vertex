use crate::drawing::*;
use crate::lpf::Lpf;
use crate::math::Vec2;
use crate::take_once::*;
use crate::ui_element::*;

pub struct Slider {
    pos: Vec2,
    dims: Vec2,
    color: Srgba,
    slider_pos: Lpf,
    hovered_animation: Lpf,
    is_hovered: bool,
    is_clicked: bool,
}

impl Slider {
    pub fn new(pos: Vec2, dims: Vec2, color: Srgba) -> Self {
        Self {
            pos,
            dims,
            color,
            slider_pos: Lpf::new(0.0, 0.0, 0.3),
            hovered_animation: Lpf::new(0.0, 0.0, 0.2),
            is_hovered: false,
            is_clicked: false,
        }
    }

    fn slider_endpoints(&self) -> (Vec2, Vec2, Vec2) {
        let start = self.pos + Vec2::new(10.0, self.dims.y / 2.0);
        let end = self.pos + Vec2::new(self.dims.x - 10.0, self.dims.y / 2.0);
        let mid = start.lerp(end, self.slider_pos.actual);
        (start, mid, end)
    }

    fn update_slider_position(&mut self, p: Vec2) {
        let (start, _, end) = self.slider_endpoints();
        let num = p - start;
        let den = end - start;
        self.slider_pos.target = (num.x / den.x).clamp(0.0, 1.0);
    }

    pub fn value(&self) -> f32 {
        self.slider_pos.target
    }
}

impl UiElement for Slider {
    fn contains(&self, p: Vec2) -> bool {
        let p = p - self.pos;
        0.0 <= p.x && p.x <= self.dims.x && 0.0 <= p.y && p.y <= self.dims.y
    }

    fn step(&mut self) {
        self.slider_pos.step();
        self.hovered_animation.target = self.is_hovered as u8 as f32;
        self.hovered_animation.step();
    }

    fn set_cursor_position(&mut self, t: &mut UiInput<Vec2>) {
        if let Some(p) = t.peek() {
            let p = *p;
            self.is_hovered = self.contains(p);
            if self.is_clicked {
                self.update_slider_position(p);
                t.reply(UiEvent::SliderValueChanged(0, self.value()));
            } else if self.is_hovered {
                t.take();
            }
        } else {
            self.is_hovered = false;
        }
    }

    fn on_left_click_down(&mut self, t: &mut UiInput<Vec2>) {
        if let Some(p) = t.peek() {
            if self.is_hovered {
                self.is_clicked = true;
                self.update_slider_position(*p);
                t.take();
            } else {
                self.is_clicked = false;
            }
        }
    }

    fn on_left_click_release(&mut self, _t: &mut UiInput<()>) {
        self.is_clicked = false;
    }

    fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    fn is_clicked(&self) -> bool {
        self.is_clicked
    }

    fn draw(&self, painter: &mut ShapePainter, text: &mut TextPainter) {
        draw_rect(
            painter,
            self.pos - Vec2::splat(5.0),
            self.dims,
            0.2,
            LIGHT_GRAY,
        );

        draw_rect(painter, self.pos, self.dims, 0.3, WHITE);

        if self.hovered_animation.actual > 0.01 {
            draw_rect(
                painter,
                self.pos,
                self.dims * self.hovered_animation.actual,
                0.31,
                WHITE.mix(&GRAY, 0.1),
            );
        }

        let (start, mid, end) = self.slider_endpoints();
        draw_line(painter, start, end, 50.0, 1.0, GRAY);

        let filled_slider_color = GRAY.mix(&self.color, self.hovered_animation.actual);

        draw_line(painter, start, mid, 51.0, 10.0, filled_slider_color);

        draw_circle(painter, mid, 52.0, 5.0, BLACK);

        text.set_position((mid + Vec2::new(0.0, 20.0)).extend(55.0));
    }
}
