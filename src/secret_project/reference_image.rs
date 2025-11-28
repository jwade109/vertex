use crate::*;

pub struct ReferenceImagePlugin;

impl Plugin for ReferenceImagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                insert_new_image,
                update_transparency,
                update_visibility,
                draw_windows,
                sync_sprite_to_window,
                insert_window_if_needed,
                open_images,
            ),
        );
        app.add_systems(FixedUpdate, step_windows);
        app.add_message::<OpenImage>();
    }
}

fn step_windows(windows: Query<&mut RefImageWindow>) {
    for mut w in windows {
        w.step();
    }
}

#[derive(Message, Debug, Deref)]
pub struct OpenImage(pub ReferenceImage);

fn open_images(
    mut commands: Commands,
    mut messages: MessageReader<OpenImage>,
    asset_server: Res<AssetServer>,
) {
    for msg in messages.read() {
        let handle = asset_server.load(format!("{}", msg.path.display()));

        commands.spawn((
            Sprite::from_image(handle),
            RefImagePath(msg.path.clone()),
            Transform::from_xyz(msg.pos.x, msg.pos.y, REF_IMAGE_Z).with_scale(Vec3::ZERO),
        ));
    }
}

fn update_transparency(app: Res<Settings>, mut query: Query<(&mut Sprite, &RefImageWindow)>) {
    for (mut s, window) in &mut query {
        let alpha = app.ref_image_alpha.lerp(
            1.0,
            window
                .hovered_animation
                .actual
                .max(window.clicked_animation.actual),
        );
        s.color = Srgba::new(1.0, 1.0, 1.0, alpha).into();
    }
}

fn update_visibility(
    state: Res<State<EditorMode>>,
    mut windows: Query<&mut Visibility, With<RefImageWindow>>,
) {
    let v = match **state {
        EditorMode::Play => Visibility::Hidden,
        _ => Visibility::Visible,
    };

    for mut vis in &mut windows {
        *vis = v;
    }
}

fn draw_windows(
    mut painter: ShapePainter,
    mut text: ResMut<TextPainter>,
    query: Query<&RefImageWindow>,
    state: Res<State<EditorMode>>,
) {
    if state.is_play() {
        return;
    }

    for window in &query {
        window.draw(&mut painter, &mut text);
    }
}

const REF_IMAGE_Z: f32 = 0.0;

fn insert_new_image(mut commands: Commands, mut msg: MessageReader<FileMessage>) -> Result {
    for msg in msg.read() {
        let (filetype, path) = if let FileMessage::Opened(filetype, path) = msg {
            (filetype, path)
        } else {
            continue;
        };

        match filetype {
            FileType::Any => (),
            FileType::Puzzle => continue,
            FileType::ReferenceImage => (),
        }

        if path.extension().map(|s| s.to_str().unwrap()) == Some("txt") {
            warn!("Tried to load puzzle as image. Fix this.");
            continue;
        }

        commands.write_message(OpenImage(ReferenceImage {
            path: path.clone(),
            pos: Vec2::ZERO,
        }));
    }

    Ok(())
}

fn insert_window_if_needed(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Sprite), Without<RefImageWindow>>,
    images: Res<Assets<Image>>,
) {
    for (e, tf, sprite) in query {
        if let Some(img) = images.get(sprite.image.id()) {
            let size = img.size().as_vec2();
            let max_elem = size.max_element();
            let size = size / max_elem * 800.0f32.min(max_elem);
            commands
                .entity(e)
                .insert(RefImageWindow::new(tf.translation.xy(), size));
        }
    }
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
                println!("Despawning failed image");
                continue;
            }
        };

        if let Some(image) = images.get(sprite.image.id()) {
            let size = image.size().as_vec2();
            tf.translation.x = window.pos.x;
            tf.translation.y = window.pos.y;
            tf.scale.x = window.dims_actual.x / size.x;
            tf.scale.y = window.dims_actual.y / size.y;
        }
    }
}

#[derive(Debug)]
struct HandleState {
    // is_hovered: bool,
    // is_clicked: bool,
    color: Srgba,
}

impl HandleState {
    fn new(color: Srgba) -> Self {
        Self {
            // is_hovered: false,
            // is_clicked: false,
            color,
        }
    }
}

#[derive(Component, Debug)]
pub struct RefImagePath(pub PathBuf);

#[derive(Component, Debug)]
pub struct RefImageWindow {
    pub pos: Vec2,
    dims_actual: Vec2,
    pub dims_target: Vec2,
    mouse_delta: Option<Vec2>,
    is_hovered: bool,
    is_clicked: bool,
    hovered_animation: Lpf,
    clicked_animation: Lpf,
    handle_states: [HandleState; 4],
    should_despawn: bool,
}

impl RefImageWindow {
    pub fn new(pos: Vec2, dims: Vec2) -> Self {
        Self {
            pos,
            dims_actual: dims * 0.9,
            dims_target: dims,
            mouse_delta: None,
            is_hovered: false,
            is_clicked: false,
            hovered_animation: Lpf::new(0.0, 0.0, 0.2),
            clicked_animation: Lpf::new(0.0, 0.0, 0.2),
            handle_states: [
                HandleState::new(RED),
                HandleState::new(WHITE),
                HandleState::new(WHITE),
                HandleState::new(WHITE),
            ],
            should_despawn: false,
        }
    }

    fn contains_basic_bb(&self, p: Vec2) -> bool {
        let p = p - (self.pos - self.dims_actual / 2.0);
        0.0 <= p.x && p.x <= self.dims_actual.x && 0.0 <= p.y && p.y <= self.dims_actual.y
    }

    fn corners(&self) -> [Vec2; 4] {
        let half = self.dims_actual / 2.0;
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
        self.hovered_animation.actual * (1.0 - self.clicked_animation.actual) * 20.0
    }

    fn contains_corners(&self, p: Vec2) -> bool {
        let r = self.handle_radius();
        self.corners().into_iter().any(|c| c.distance(p) <= r)
    }

    pub fn step(&mut self) {
        self.hovered_animation.target = self.is_hovered as u8 as f32;
        self.hovered_animation.step();
        self.clicked_animation.target = self.is_clicked as u8 as f32;
        self.clicked_animation.step();
        self.dims_actual += (self.dims_target - self.dims_actual) * self.hovered_animation.alpha;
    }

    pub fn should_despawn(&self) -> bool {
        self.should_despawn
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
        } else if self.is_hovered && input.is_right_pressed() {
            self.should_despawn = true;
        }
    }

    fn draw(&self, painter: &mut ShapePainter, _text: &mut TextPainter) {
        let extra_w = 15.0 * self.clicked_animation.actual;
        let anim_hw = self.dims_actual / 2.0 + Vec2::splat(extra_w);
        let t = 2.0 + self.hovered_animation.actual * 4.0;
        let color = GRAY.mix(&BLACK, self.hovered_animation.actual);
        draw_rect(
            painter,
            self.pos - anim_hw,
            anim_hw * 2.0,
            t,
            color,
            REF_IMAGE_BORDER_Z,
        );

        let r = self.handle_radius();
        for (corner, handle) in self.handles() {
            fill_circle(painter, corner, 100.0, r, handle.color);
            draw_circle(painter, corner, 100.0, r, 3.0, BLACK);
        }
    }
}
