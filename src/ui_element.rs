use crate::math::Vec2;
use crate::take_once::TakeOnce;
pub use crate::text::TextPainter;
pub use bevy::color::palettes::css::*;
use bevy::prelude::*;
pub use bevy_vector_shapes::prelude::ShapePainter;

pub trait UiElement: Send + Sync {
    fn contains(&self, p: Vec2) -> bool;

    fn step(&mut self, commands: &mut Commands);

    fn id(&self) -> &str;

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

    fn z_index(&self) -> f32 {
        0.0
    }

    fn is_clicked(&self) -> bool {
        false
    }

    fn draw(&self, painter: &mut ShapePainter, text: &mut TextPainter);
}
