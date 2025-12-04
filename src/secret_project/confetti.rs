use crate::secret_project::*;

pub struct ConfettiPlugin;

impl Plugin for ConfettiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(VictoryScreen), on_puzzle_complete);
        app.add_systems(FixedUpdate, update_confetti);
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct ConfettiVelocity {
    linear: Vec2,
    angular: f32,
    scale: f32,
}

impl ConfettiVelocity {
    fn random(scale: f32) -> Self {
        let v = randvec(40.0, 500.0);
        let a = random(0.25, 0.75) * std::f32::consts::PI;
        let v_up = Vec2::from_angle(a) * random(0.25, 1.0) * random(75.0, 250.0);
        Self {
            linear: (v + v_up) * scale,
            angular: random(-14.0, 14.0),
            scale,
        }
    }
}

fn on_puzzle_complete(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    camera: Single<&Transform, With<Camera>>,
) {
    commands.write_message(SoundEffect::Victory);

    for _ in 0..300 {
        let spread = 5.0 * camera.scale.x;
        let p = camera.translation.with_z(0.0)
            + Vec3::new(
                random(-spread, spread),
                random(-spread, spread),
                random(-spread, spread),
            );
        let a = random(0.0, 2.0 * std::f32::consts::PI);
        let l = random(20.0, 80.0) * camera.scale.x;
        let w = random(20.0, 80.0) * camera.scale.x;
        let mesh = Mesh2d(meshes.add(Rectangle::from_size(Vec2::new(l, w))));
        let color = LinearRgba {
            red: rand(),
            green: rand(),
            blue: rand(),
            alpha: 1.0,
        };
        let mat = MeshMaterial2d(materials.add(ColorMaterial::from_color(color)));
        let tf = Transform::from_translation(p).with_rotation(Quat::from_axis_angle(Vec3::Z, a));
        commands.spawn((tf, mesh, mat, ConfettiVelocity::random(camera.scale.x)));
    }
}

fn update_confetti(
    mut commands: Commands,
    confettis: Query<(Entity, &mut ConfettiVelocity, &mut Transform)>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();
    for (e, mut vel, mut tf) in confettis {
        tf.translation += vel.linear.extend(0.0) * dt;
        tf.rotate_axis(Dir3::Z, vel.angular * dt);

        vel.linear.y -= 400.0 * vel.scale * dt;
        vel.angular *= 0.98;

        tf.scale *= 0.99;

        if tf.scale.x < 0.1 {
            commands.entity(e).despawn();
        }
    }
}
