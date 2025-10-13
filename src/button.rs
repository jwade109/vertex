use crate::app::VertexApp;
use crate::math::*;
use crate::take_once::*;
use crate::ui_element::*;
use bevy::color::Srgba;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

#[derive(Component, Debug, Clone)]
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
    pub fn new(text: impl Into<String>, pos: Vec2) -> Self {
        Button {
            text: text.into(),
            pos,
            dims: Vec2::new(160.0, 40.0),
            hover_animation: Lpf::new(0.0, 0.0, 0.2),
            clicked_animation: Lpf::new(0.0, 0.0, 0.2),
            color: Srgba::new(rand(), rand(), rand(), 1.0),
            is_hover: false,
            is_clicked: false,
        }
    }

    pub fn contains(&self, p: Vec2) -> bool {
        let p = p - self.pos;
        0.0 <= p.x && p.x <= self.dims.x && 0.0 <= p.y && p.y <= self.dims.y
    }

    pub fn step(&mut self) {
        self.hover_animation.target = self.is_hover as u8 as f32;
        self.hover_animation.step();
        self.clicked_animation.target = self.is_clicked() as u8 as f32;
        self.clicked_animation.step();
    }

    pub fn set_cursor_position(&mut self, t: &mut TakeOnce<Vec2>) {
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

    pub fn is_hovered(&self) -> bool {
        self.is_hover
    }

    pub fn on_input(&mut self, input: &InputMessage) {
        if self.is_hovered() && input.is_left_pressed() {
            self.on_left_click_pressed();
        } else if self.is_clicked() && input.is_left_released() {
            self.on_left_click_release();
        }
    }

    pub fn on_left_click_pressed(&mut self) {
        if self.is_hover {
            self.is_clicked = true;
        } else {
            self.is_clicked = false;
        }
    }

    pub fn on_left_click_release(&mut self) {
        self.is_clicked = false;
    }

    pub fn is_clicked(&self) -> bool {
        self.is_clicked
    }

    pub fn draw(&self, painter: &mut ShapePainter, text: &mut TextPainter) {
        painter.reset();

        let c1 = WHITE;

        let offset = Vec2::splat(5.0);

        painter.set_color(c1);
        painter.set_translation((self.pos + self.dims / 2.0).extend(0.28));
        painter.rect(self.dims);

        painter.set_color(LIGHT_GRAY);
        painter.set_translation((self.pos + self.dims / 2.0 - offset).extend(0.03));
        painter.rect(self.dims);

        let c2 = self.color.with_alpha(0.7);
        let anim_dims = self
            .dims
            .with_x((self.dims.x * self.hover_animation.actual).max(7.0));
        painter.set_color(c2);
        painter.set_translation((self.pos + anim_dims / 2.0).extend(0.29));
        painter.rect(anim_dims);

        if self.clicked_animation.actual > 0.01 {
            let c2 = self.color;
            let anim_dims = self
                .dims
                .with_x(self.dims.x * self.clicked_animation.actual);
            painter.set_color(c2);
            painter.set_translation((self.pos + anim_dims / 2.0).extend(0.29));
            painter.rect(anim_dims);
        }

        text.set_position((self.pos + self.dims / 2.0).extend(100.0));
        text.set_height(self.dims.y * 0.7);
        text.set_color(BLACK.with_alpha(1.0));
        text.text(self.text.clone());
    }
}

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(
                Update,
                (
                    draw_buttons,
                    step_buttons,
                    translate_cursor_moved,
                    translate_cursor_entered,
                    translate_cursor_left,
                    translate_mouse_buttons,
                    cursor_to_buttons,
                    on_generic_input,
                ),
            )
            .add_message::<InputMessage>();
    }
}

fn startup(mut commands: Commands) {
    let button_text = [
        "Load",
        "Save",
        "Picker",
        "Normalize",
        "Whatever",
        "Whocares",
    ];

    for (i, text) in button_text.into_iter().enumerate() {
        let pos = Vec2::new(-900.0, i as f32 * 60.0);
        let button = Button::new(text, pos);
        commands.spawn(button);
    }
}

fn draw_buttons(mut painter: ShapePainter, mut text: ResMut<TextPainter>, query: Query<&Button>) {
    for button in &query {
        button.draw(&mut painter, &mut text);
    }
}

fn translate_cursor_moved(
    mut cursor: MessageReader<CursorMoved>,
    mut out: MessageWriter<InputMessage>,
) {
    for event in cursor.read() {
        let event = InputMessage::new(event.clone());
        out.write(event);
    }
}

fn translate_cursor_entered(
    mut cursor: MessageReader<CursorEntered>,
    mut out: MessageWriter<InputMessage>,
) {
    for event in cursor.read() {
        let event = InputMessage::new(event.clone());
        out.write(event);
    }
}

fn translate_cursor_left(
    mut cursor: MessageReader<CursorLeft>,
    mut out: MessageWriter<InputMessage>,
) {
    for event in cursor.read() {
        let event = InputMessage::new(event.clone());
        out.write(event);
    }
}

fn translate_mouse_buttons(
    mut mouse: MessageReader<MouseButtonInput>,
    mut out: MessageWriter<InputMessage>,
) {
    for event in mouse.read() {
        let event = InputMessage::new(event.clone());
        out.write(event);
    }
}

fn on_generic_input(mut query: Query<&mut Button>, mut msg: MessageReader<InputMessage>) {
    for input in msg.read() {
        for mut button in &mut query {
            button.on_input(input);
        }
    }
}

fn step_buttons(mut query: Query<&mut Button>) {
    for mut button in &mut query {
        button.step();
    }
}

fn cursor_to_buttons(app: Res<VertexApp>, mut query: Query<&mut Button>) {
    for mut button in &mut query {
        let mut once = TakeOnce::from_option(app.mouse_pos);
        button.set_cursor_position(&mut once);
    }
}
