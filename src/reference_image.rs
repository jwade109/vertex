use crate::app::VertexApp;
use crate::drawing::*;
use crate::file_open_system::*;
use crate::math::*;
use crate::take_once::TakeOnce;
use crate::text::TextPainter;
use crate::ui_element::UiElement;
use bevy::prelude::*;

pub struct ReferenceImagePlugin;

impl Plugin for ReferenceImagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                insert_new_image,
                update_transparency,
                sync_to_sprites,
                read_window_events,
            ),
        )
        .add_message::<WindowUpdate>();
    }
}

fn update_transparency(app: Res<VertexApp>, mut query: Query<&mut Sprite>) {
    for mut s in &mut query {
        s.color = Srgba::new(1.0, 1.0, 1.0, app.ref_image_alpha).into();
    }
}

const REF_IMAGE_Z: f32 = 0.0;

fn read_window_events(mut msg: MessageReader<WindowUpdate>) {
    for msg in msg.read() {
        dbg!(msg);
    }
}

fn sync_to_sprites(
    mut updates: MessageReader<WindowUpdate>,
    mut query: Query<(Entity, &mut Transform, &Sprite)>,
    assets: Res<Assets<Image>>,
) {
    for update in updates.read() {
        if let Ok((_, mut tf, s)) = query.get_mut(update.target) {
            if let Some(img) = assets.get(s.image.id()) {
                let size = img.size().as_vec2();
                tf.translation = (update.pos + update.size / 2.0).extend(REF_IMAGE_Z);
                tf.scale.x = update.size.x / size.x;
                tf.scale.y = update.size.y / size.y;
            }
        }
    }
}

fn insert_new_image(
    mut commands: Commands,
    mut msg: MessageReader<FileMessage>,
    mut app: ResMut<VertexApp>,
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

        let id = commands
            .spawn((
                Sprite::from_image(handle),
                Transform::from_xyz(x, y, REF_IMAGE_Z).with_scale(Vec3::splat(0.5)),
            ))
            .id();

        let window = RefImageWindow::new(id, Vec2::new(x, y));
        app.buttons.push(Box::new(window));
    }

    Ok(())
}

#[derive(Component)]
struct RefImageWindow {
    target: Entity,
    pos: Vec2,
    old_pos: Vec2,
    dims: Vec2,
    mouse_delta: Option<Vec2>,
    is_hover: bool,
    animation: Lpf,
}

#[derive(Message, Debug, Clone, Copy)]
struct WindowUpdate {
    target: Entity,
    pos: Vec2,
    size: Vec2,
}

impl RefImageWindow {
    pub fn new(target: Entity, pos: Vec2) -> Self {
        Self {
            target,
            pos,
            old_pos: pos,
            dims: Vec2::new(800.0, 600.0),
            mouse_delta: None,
            is_hover: false,
            animation: Lpf::new(0.0, 0.0, 0.2),
        }
    }
}

impl UiElement for RefImageWindow {
    fn contains(&self, p: Vec2) -> bool {
        let p = p - self.pos;
        0.0 <= p.x && p.x <= self.dims.x && 0.0 <= p.y && p.y <= self.dims.y
    }

    fn step(&mut self, commands: &mut Commands) {
        self.animation.target = self.is_clicked() as u8 as f32;
        self.animation.step();

        if self.pos != self.old_pos {
            commands.write_message(WindowUpdate {
                target: self.target,
                pos: self.pos,
                size: self.dims,
            });
            self.old_pos = self.pos;
        }
    }

    fn id(&self) -> &str {
        "Reference Image"
    }

    fn set_cursor_position(&mut self, t: &mut TakeOnce<Vec2>) {
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

    fn on_left_click_down(&mut self, t: &mut TakeOnce<Vec2>) {
        if let Some(p) = t.peek() {
            if self.is_hover {
                self.mouse_delta = Some(*p - self.pos);
                t.take();
            }
        } else {
            self.mouse_delta = None;
        }
    }

    fn on_left_click_release(&mut self, _t: &mut TakeOnce<()>) {
        self.mouse_delta = None;
    }

    fn is_hovered(&self) -> bool {
        self.is_hover
    }

    fn is_clicked(&self) -> bool {
        self.mouse_delta.is_some()
    }

    fn draw(&self, painter: &mut ShapePainter, _text: &mut TextPainter) {
        draw_rect(
            painter,
            self.pos,
            self.dims,
            if self.is_hover { TEAL } else { GRAY },
        );

        draw_rect(painter, self.pos, self.dims * self.animation.actual, BLUE);
    }
}
