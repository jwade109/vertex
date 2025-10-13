use crate::app::VertexApp;
use crate::drawing::*;
use crate::file_open_system::*;
use crate::math::*;
use crate::take_once::*;
use crate::text::TextPainter;
use bevy::prelude::*;

pub struct ReferenceImagePlugin;

impl Plugin for ReferenceImagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                insert_new_image,
                update_transparency,
                draw_windows,
                sync_sprite_to_window,
            ),
        );
    }
}

fn update_transparency(app: Res<VertexApp>, mut query: Query<&mut Sprite>) {
    for mut s in &mut query {
        s.color = Srgba::new(1.0, 1.0, 1.0, app.ref_image_alpha).into();
    }
}

fn draw_windows(
    mut painter: ShapePainter,
    mut text: ResMut<TextPainter>,
    query: Query<&RefImageWindow>,
) {
    for window in &query {
        window.draw(&mut painter, &mut text);
    }
}

const REF_IMAGE_Z: f32 = 0.0;

fn insert_new_image(
    mut commands: Commands,
    mut msg: MessageReader<FileMessage>,
    asset_server: Res<AssetServer>,
) -> Result {
    for msg in msg.read() {
        let path = if let FileMessage::Opened(FileType::ReferenceImage, path) = msg {
            path
        } else {
            continue;
        };

        let handle = asset_server.load(format!("{}", path.display()));

        let x = random(-100.0, 100.0);
        let y = random(-100.0, 100.0);

        commands.spawn((
            Sprite::from_image(handle),
            Transform::from_xyz(x, y, REF_IMAGE_Z).with_scale(Vec3::splat(0.5)),
            RefImageWindow::new(Vec2::new(x, y)),
        ));
    }

    Ok(())
}

fn sync_sprite_to_window(
    mut query: Query<(&RefImageWindow, &mut Transform, &Sprite)>,
    images: Res<Assets<Image>>,
) {
    for (window, mut tf, sprite) in &mut query {
        if let Some(image) = images.get(sprite.image.id()) {
            let size = image.size().as_vec2();
            tf.translation.x = window.pos.x + window.dims.x / 2.0;
            tf.translation.y = window.pos.y + window.dims.y / 2.0;
            tf.scale.x = window.dims.x / size.x;
            tf.scale.y = window.dims.y / size.y;
        }
    }
}

#[derive(Component)]
pub struct RefImageWindow {
    pos: Vec2,
    dims: Vec2,
    mouse_delta: Option<Vec2>,
    is_hover: bool,
    is_clicked: bool,
    animation: Lpf,
}

impl RefImageWindow {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            dims: Vec2::new(800.0, 600.0),
            mouse_delta: None,
            is_hover: false,
            is_clicked: false,
            animation: Lpf::new(0.0, 0.0, 0.2),
        }
    }

    fn contains(&self, p: Vec2) -> bool {
        let p = p - self.pos;
        0.0 <= p.x && p.x <= self.dims.x && 0.0 <= p.y && p.y <= self.dims.y
    }

    pub fn step(&mut self) {
        self.animation.target = self.is_clicked as u8 as f32;
        self.animation.step();
    }

    pub fn set_cursor_position(&mut self, t: &mut TakeOnce<Vec2>) {
        if let Some(p) = t.peek() {
            let p = *p;
            self.is_hover = self.contains(p);
            if self.is_hover {
                t.take();
                if self.is_clicked {
                    if let Some(q) = self.mouse_delta {
                        self.pos = p - q;
                    }
                } else {
                    self.mouse_delta = Some(p - self.pos);
                }
            }
        } else {
            self.is_hover = false;
        }
    }

    fn on_left_click_pressed(&mut self) {
        self.is_clicked = self.is_hover;
    }

    fn on_left_click_release(&mut self) {
        self.is_clicked = false;
    }

    fn is_hovered(&self) -> bool {
        self.is_hover
    }

    pub fn on_input(&mut self, input: &InputMessage) {
        if self.is_hovered() && input.is_left_pressed() {
            self.on_left_click_pressed();
        } else if self.is_clicked && input.is_left_released() {
            self.on_left_click_release();
        }
    }

    fn draw(&self, painter: &mut ShapePainter, _text: &mut TextPainter) {
        draw_rect(
            painter,
            self.pos,
            self.dims,
            if self.is_hover { TEAL } else { BLACK },
        );

        fill_rect(painter, self.pos, self.dims * self.animation.actual, BLUE.with_alpha(0.3));
    }
}
