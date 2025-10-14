use crate::math::Vec2;
use crate::take_once::TakeOnce;
pub use crate::text::TextPainter;
pub use bevy::color::*;
pub use bevy_vector_shapes::prelude::ShapePainter;

#[derive(Debug)]
pub enum UiEvent<Id> {
    SliderValueChanged(Id, f32),
    OnHover(Id),
}

pub type UiInput<T> = TakeOnce<T, UiEvent<u32>>;

pub trait UiElement: Send + Sync {
    fn contains(&self, p: Vec2) -> bool;

    fn step(&mut self);

    fn set_cursor_position(&mut self, t: &mut UiInput<Vec2>);

    fn on_left_click_down(&mut self, _t: &mut UiInput<Vec2>);

    fn on_left_click_release(&mut self, _t: &mut UiInput<()>);

    fn is_hovered(&self) -> bool;

    fn is_clicked(&self) -> bool;

    fn draw(&self, painter: &mut ShapePainter, text: &mut TextPainter);
}
