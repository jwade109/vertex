use crate::secret_project::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                on_keys.run_if(camera_is_moveable),
                (on_input_tick, middle_click_pan.run_if(camera_is_moveable)).chain(),
                insert_component_data,
            ),
        )
        .insert_resource(PanState::default())
        .add_systems(FixedUpdate, camera_physics);
    }
}

#[derive(Component, Debug, Default)]
struct CameraController {
    // linear_vel: Vec2,
    zoom_vel: f32,
}

// TODO this doesn't belong here. sue me
fn on_input_tick(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut cursor: ResMut<CursorState>,
    mut puzzle: Single<&mut Puzzle>,
    camera: Single<(&Camera, &GlobalTransform)>,
    app: Res<Settings>,
) {
    let (camera, camera_transform) = *camera;

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        cursor.mouse_pos = Some(world_position);
    } else {
        cursor.mouse_pos = None;
    }

    // keyboard presses
    if keys.just_pressed(KeyCode::KeyQ) {
        if keys.pressed(KeyCode::ControlLeft) {
            commands.write_message(Quantize(app.n_colors));
            commands.write_message(SoundEffect::UiThreePop);
        } else {
            if let Some(p) = cursor.get() {
                puzzle.add_point(p);
                commands.write_message(SoundEffect::LightPop);
            }
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        commands.write_message(AppExit::Success);
    }
}

fn insert_component_data(
    mut commands: Commands,
    query: Query<Entity, (With<Camera>, Without<CameraController>)>,
) {
    for e in query {
        commands.entity(e).insert(CameraController::default());
    }
}

fn camera_physics(
    mut camera: Query<(&mut Transform, &mut CameraController)>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();
    for (mut tf, mut ctrl) in &mut camera {
        // let sx = tf.scale.x;
        // tf.translation += ctrl.linear_vel.extend(0.0) * dt * sx;
        tf.scale.x *= 1.0 + ctrl.zoom_vel * dt;
        tf.scale.x = tf.scale.x.clamp(0.01, 4.0);

        tf.scale.y = tf.scale.x;

        // ctrl.linear_vel *= 0.87;
        ctrl.zoom_vel *= 0.87;

        tf.translation.z = 100.0;
    }
}

fn on_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut query: Query<&mut CameraController>,
) {
    // let speed = 1400.0;
    let zoom_speed = 5.0;

    let is_key = |key: KeyCode| {
        for ex in [KeyCode::ControlLeft, KeyCode::ControlRight] {
            if keys.pressed(ex) {
                return false;
            }
        }
        keys.pressed(key)
    };

    for mut ctrl in &mut query {
        // ctrl.linear_vel.y = if is_key(KeyCode::KeyW) {
        //     speed
        // } else if is_key(KeyCode::KeyS) {
        //     -speed
        // } else {
        //     ctrl.linear_vel.y
        // };

        // ctrl.linear_vel.x = if is_key(KeyCode::KeyD) {
        //     speed
        // } else if is_key(KeyCode::KeyA) {
        //     -speed
        // } else {
        //     ctrl.linear_vel.x
        // };

        ctrl.zoom_vel = if is_key(KeyCode::Minus) {
            zoom_speed
        } else if is_key(KeyCode::Equal) {
            -zoom_speed
        } else {
            ctrl.zoom_vel
        };

        for event in mouse_wheel.read() {
            let zoom_speed = match event.unit {
                // desktop mice?
                MouseScrollUnit::Line => {
                    debug!(
                        "Scroll (line units): vertical: {}, horizontal: {}",
                        event.y, event.x
                    );
                    zoom_speed * event.y.abs()
                }
                // trackpad?
                MouseScrollUnit::Pixel => {
                    debug!(
                        "Scroll (pixel units): vertical: {}, horizontal: {}",
                        event.y, event.x
                    );
                    zoom_speed * event.y.abs()
                }
            };

            let delta = -Vec2::new(event.x, event.y);
            ctrl.zoom_vel = if delta.y > 0.0 {
                zoom_speed
            } else if delta.y < 0.0 {
                -zoom_speed
            } else {
                ctrl.zoom_vel
            };
        }
    }
}

#[derive(Resource, Default)]
struct PanState {
    previous: Option<Vec2>,
}

fn middle_click_pan(
    pos: Res<CursorState>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut panstate: ResMut<PanState>,
    mut camera: Single<&mut Transform, With<Camera>>,
) {
    let pos = match pos.get() {
        Some(p) => p,
        None => {
            panstate.previous = None;
            return;
        }
    };

    if mouse.just_pressed(MouseButton::Middle) {
        panstate.previous = Some(pos);
    }

    if mouse.just_released(MouseButton::Middle) {
        panstate.previous = None;
    }

    if mouse.pressed(MouseButton::Middle) {
        if let Some(prev) = panstate.previous {
            let delta = pos - prev;
            let new_pos = camera.translation.xy() - delta;
            camera.translation.x = new_pos.x;
            camera.translation.y = new_pos.y;
        }
    }
}
