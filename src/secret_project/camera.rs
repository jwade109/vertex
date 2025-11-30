use crate::secret_project::*;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (on_keys.run_if(camera_is_moveable), insert_component_data),
        )
        .add_systems(FixedUpdate, camera_physics);
    }
}

#[derive(Component, Debug, Default)]
struct CameraController {
    linear_vel: Vec2,
    zoom_vel: f32,
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
        let sx = tf.scale.x;
        tf.translation += ctrl.linear_vel.extend(0.0) * dt * sx;
        tf.scale.x *= 1.0 + ctrl.zoom_vel * dt;
        tf.scale.x = tf.scale.x.clamp(0.01, 4.0);

        tf.scale.y = tf.scale.x;

        ctrl.linear_vel *= 0.87;
        ctrl.zoom_vel *= 0.87;

        tf.translation.z = 100.0;
    }
}

fn on_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut query: Query<&mut CameraController>,
) {
    let speed = 1400.0;
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
        ctrl.linear_vel.y = if is_key(KeyCode::KeyW) {
            speed
        } else if is_key(KeyCode::KeyS) {
            -speed
        } else {
            ctrl.linear_vel.y
        };

        ctrl.linear_vel.x = if is_key(KeyCode::KeyD) {
            speed
        } else if is_key(KeyCode::KeyA) {
            -speed
        } else {
            ctrl.linear_vel.x
        };

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
