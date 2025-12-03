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
                step_windows_active.run_if(in_state(AppState::Editing {
                    mode: EditorMode::Images,
                })),
                step_windows_inactive.run_if(not(in_state(AppState::Editing {
                    mode: EditorMode::Images,
                }))),
            ),
        );
        app.add_message::<OpenReferenceImage>();
    }
}

fn step_windows_active(
    windows: Query<&mut RefImageWindow>,
    cursor: Res<CursorState>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    for mut w in windows {
        if let Some(p) = cursor.get() {
            w.is_hovered = w.contains_basic_bb(p);

            if w.is_hovered {
                if mouse.just_pressed(MouseButton::Left) {
                    w.is_clicked = true;
                }
                if !mouse.pressed(MouseButton::Left) {
                    w.is_clicked = false;
                }
            } else {
                w.is_clicked = false;
            }

            if w.is_clicked {
                if let Some(delta) = w.mouse_delta {
                    w.pos = p - delta;
                }
            } else {
                w.mouse_delta = Some(p - w.pos);
            }
        } else {
            w.is_hovered = false;
            w.is_clicked = false;
            w.mouse_delta = None;
        }
    }
}

fn step_windows_inactive(windows: Query<&mut RefImageWindow>) {
    for mut w in windows {
        w.is_hovered = false;
        w.is_clicked = false;
        w.mouse_delta = None;
    }
}

#[derive(Message, Debug)]
pub struct OpenReferenceImage {
    pub path: PathBuf,
    pub pos: Vec2,
}

fn open_images(
    mut commands: Commands,
    mut messages: MessageReader<OpenReferenceImage>,
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
            (window.is_hovered as u8 as f32).max(window.is_clicked as u8 as f32),
        );
        s.color = Srgba::new(1.0, 1.0, 1.0, alpha).into();
    }
}

fn update_visibility(
    state: Res<State<AppState>>,
    mut windows: Query<&mut Visibility, With<RefImageWindow>>,
) {
    let v = match **state {
        AppState::Editing { .. } => Visibility::Visible,
        _ => Visibility::Hidden,
    };

    for mut vis in &mut windows {
        *vis = v;
    }
}

fn draw_windows(
    mut painter: ShapePainter,
    query: Query<&RefImageWindow>,
    state: Res<State<AppState>>,
) {
    if !state.is_editor() {
        return;
    }

    for window in &query {
        window.draw(&mut painter);
    }
}

const REF_IMAGE_Z: f32 = 0.0;

fn insert_new_image(mut commands: Commands, mut msg: MessageReader<FileMessage>) -> Result {
    for msg in msg.read() {
        let path = if let FileMessage::Opened(_, path) = msg {
            path
        } else {
            continue;
        };

        if path.extension().map(|s| s.to_str().unwrap()) == Some("txt") {
            warn!("Tried to load puzzle as image. Fix this.");
            continue;
        }

        commands.write_message(OpenReferenceImage {
            path: path.clone(),
            pos: Vec2::ZERO,
        });
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
            tf.scale.x = window.dims.x / size.x;
            tf.scale.y = window.dims.y / size.y;
        }
    }
}

#[derive(Component, Debug)]
pub struct RefImagePath(pub PathBuf);

#[derive(Component, Debug)]
pub struct RefImageWindow {
    pub pos: Vec2,
    pub dims: Vec2,
    mouse_delta: Option<Vec2>,
    is_hovered: bool,
    is_clicked: bool,
    should_despawn: bool,
}

impl RefImageWindow {
    pub fn new(pos: Vec2, dims: Vec2) -> Self {
        Self {
            pos,
            dims: dims,
            mouse_delta: None,
            is_hovered: false,
            is_clicked: false,
            should_despawn: false,
        }
    }

    fn contains_basic_bb(&self, p: Vec2) -> bool {
        let p = p - (self.pos - self.dims / 2.0);
        0.0 <= p.x && p.x <= self.dims.x && 0.0 <= p.y && p.y <= self.dims.y
    }

    pub fn should_despawn(&self) -> bool {
        self.should_despawn
    }

    fn draw(&self, painter: &mut ShapePainter) {
        let extra_w = 15.0 * self.is_clicked as u8 as f32;
        let anim_hw = self.dims / 2.0 + Vec2::splat(extra_w);
        let t = 2.0 + self.is_hovered as u8 as f32 * 4.0;
        let color = GRAY.mix(&BLACK, self.is_hovered as u8 as f32);
        draw_rect(
            painter,
            self.pos - anim_hw,
            anim_hw * 2.0,
            t,
            color,
            REF_IMAGE_BORDER_Z,
        );
    }
}
