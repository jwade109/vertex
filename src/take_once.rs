use bevy::input::mouse::*;
use bevy::input::*;
use bevy::prelude::*;

pub struct TakeOnce<T>(Option<T>);

impl<T> TakeOnce<T> {
    pub fn new(val: T) -> Self {
        Self(Some(val))
    }

    pub fn from_option(val: Option<T>) -> Self {
        Self(val)
    }

    pub fn take(&mut self) -> Option<T> {
        let ret = self.0.take();
        self.0 = None;
        ret
    }

    pub fn peek(&self) -> Option<&T> {
        self.0.as_ref()
    }
}

#[derive(Message, Debug, Clone)]
pub struct InputMessage {
    event: InputEvent,
    should_propagate: bool,
}

impl InputMessage {
    pub fn new(event: impl Into<InputEvent>) -> Self {
        Self {
            event: event.into(),
            should_propagate: true,
        }
    }

    pub fn event(&self) -> &InputEvent {
        &self.event
    }

    pub fn dont_propagate(&mut self) {
        self.should_propagate = false;
    }

    pub fn should_propagate(&self) -> bool {
        self.should_propagate
    }

    pub fn is_left_released(&self) -> bool {
        match self.event() {
            InputEvent::MouseButton(i) => match (i.button, i.state) {
                (MouseButton::Left, ButtonState::Released) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_left_pressed(&self) -> bool {
        match self.event() {
            InputEvent::MouseButton(i) => match (i.button, i.state) {
                (MouseButton::Left, ButtonState::Pressed) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_right_released(&self) -> bool {
        match self.event() {
            InputEvent::MouseButton(i) => match (i.button, i.state) {
                (MouseButton::Right, ButtonState::Released) => true,
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_right_pressed(&self) -> bool {
        match self.event() {
            InputEvent::MouseButton(i) => match (i.button, i.state) {
                (MouseButton::Right, ButtonState::Pressed) => true,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    Moved(CursorMoved),
    Left(CursorLeft),
    Entered(CursorEntered),
    MouseButton(MouseButtonInput),
}

impl Into<InputEvent> for CursorMoved {
    fn into(self) -> InputEvent {
        InputEvent::Moved(self)
    }
}

impl Into<InputEvent> for CursorEntered {
    fn into(self) -> InputEvent {
        InputEvent::Entered(self)
    }
}

impl Into<InputEvent> for CursorLeft {
    fn into(self) -> InputEvent {
        InputEvent::Left(self)
    }
}

impl Into<InputEvent> for MouseButtonInput {
    fn into(self) -> InputEvent {
        InputEvent::MouseButton(self)
    }
}
