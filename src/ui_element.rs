use crate::math::Vec2;
use crate::take_once::TakeOnce;
pub use crate::text::TextPainter;
pub use bevy::color::palettes::css::*;
pub use bevy_vector_shapes::prelude::ShapePainter;

pub trait UiElement: Send + Sync {
    fn contains(&self, p: Vec2) -> bool;

    fn step(&mut self);

    fn set_cursor_position(&mut self, t: &mut TakeOnce<Vec2>);

    fn on_left_click_down(&mut self, _t: &mut TakeOnce<Vec2>) {
        // stub
    }

    fn on_left_click_release(&mut self, _t: &mut TakeOnce<()>) {
        // stub
    }

    #[allow(unused)]
    fn is_hovered(&self) -> bool {
        false
    }

    fn is_clicked(&self) -> bool {
        false
    }

    fn draw(&self, painter: &mut ShapePainter, text: &mut TextPainter);
}
