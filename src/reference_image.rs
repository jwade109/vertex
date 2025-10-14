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
    mut commands: Commands,
    mut query: Query<(Entity, &RefImageWindow, &mut Transform, &Sprite)>,
    images: Res<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    use bevy::asset::LoadState;

    for (e, window, mut tf, sprite) in &mut query {
        let state = asset_server.load_state(sprite.image.id());
        match state {
            LoadState::NotLoaded => (),
            LoadState::Loading => (),
            LoadState::Loaded => (),
            LoadState::Failed(_) => {
                commands.entity(e).despawn();
                continue;
            }
        };

        if let Some(image) = images.get(sprite.image.id()) {
            let size = image.size().as_vec2();
            tf.translation.x = window.pos.x;
            tf.translation.y = window.pos.y;
            tf.scale.x = window.dims.x / size.x;
            tf.scale.y = window.dims.y / size.y;
        }
    }
}

struct HandleState {
    is_hovered: bool,
    is_clicked: bool,
    color: Srgba,
}

impl HandleState {
    fn new(color: Srgba) -> Self {
        Self {
            is_hovered: false,
            is_clicked: false,
            color,
        }
    }
}

#[derive(Component)]
pub struct RefImageWindow {
    pos: Vec2,
    dims: Vec2,
    mouse_delta: Option<Vec2>,
    is_hovered: bool,
    is_clicked: bool,
    hovered_animation: Lpf,
    handle_states: [HandleState; 4],
}

impl RefImageWindow {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            dims: Vec2::new(800.0, 600.0),
            mouse_delta: None,
            is_hovered: false,
            is_clicked: false,
            hovered_animation: Lpf::new(0.0, 0.0, 0.2),
            handle_states: [
                HandleState::new(RED),
                HandleState::new(WHITE),
                HandleState::new(WHITE),
                HandleState::new(WHITE),
            ],
        }
    }

    fn contains_basic_bb(&self, p: Vec2) -> bool {
        let p = p - (self.pos - self.dims / 2.0);
        0.0 <= p.x && p.x <= self.dims.x && 0.0 <= p.y && p.y <= self.dims.y
    }

    fn corners(&self) -> [Vec2; 4] {
        let half = self.dims / 2.0;
        let flipped = half.with_x(-half.x);

        [
            self.pos + half,
            self.pos + flipped,
            self.pos - half,
            self.pos - flipped,
        ]
    }

    fn handles(&self) -> impl Iterator<Item = (Vec2, &HandleState)> + use<'_> {
        self.corners().into_iter().zip(self.handle_states.iter())
    }

    fn handle_radius(&self) -> f32 {
        self.hovered_animation.actual * 20.0
    }

    fn contains_corners(&self, p: Vec2) -> bool {
        let r = self.handle_radius();
        self.corners().into_iter().any(|c| c.distance(p) <= r)
    }

    pub fn step(&mut self) {
        self.hovered_animation.target = self.is_hovered as u8 as f32;
        self.hovered_animation.step();
    }

    pub fn set_cursor_position(&mut self, t: &mut TakeOnce<Vec2>) {
        if let Some(p) = t.peek() {
            let p = *p;
            self.is_hovered = self.contains_basic_bb(p) || self.contains_corners(p);
            if self.is_hovered {
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
            self.is_hovered = false;
        }
    }

    fn on_left_click_pressed(&mut self) {
        self.is_clicked = self.is_hovered;
    }

    fn on_left_click_release(&mut self) {
        self.is_clicked = false;
    }

    pub fn on_input(&mut self, input: &mut InputMessage) {
        if self.is_hovered && input.is_left_pressed() {
            self.on_left_click_pressed();
            input.dont_propagate();
        } else if self.is_clicked && input.is_left_released() {
            self.on_left_click_release();
            input.dont_propagate();
        }
    }

    fn draw(&self, painter: &mut ShapePainter, _text: &mut TextPainter) {
        draw_rect(
            painter,
            self.pos - self.dims / 2.0,
            self.dims,
            4.0,
            if self.is_hovered { TEAL } else { BLACK },
        );

        let r = self.handle_radius();
        for (corner, handle) in self.handles() {
            draw_circle(painter, corner, 100.0, r, handle.color);
            draw_hollow_circle(painter, corner, 100.0, r, 3.0, BLACK);
        }
    }
}
