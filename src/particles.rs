use crate::app::VertexApp;
use crate::cursor::*;
use crate::drawing::*;
use crate::grid::*;
use crate::math::random;
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                add_particles,
                update_ripples,
                update_grid_particles,
                spawn_random_ripples,
            ),
        );
    }
}

#[derive(Component)]
struct Ripple(Vec2, f32);

#[derive(Component)]
struct GridParticle(Vec2, Vec2, f32);

fn add_particles(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    cursor: Res<CursorState>,
) {
    let p = if let Some(p) = cursor.mouse_pos {
        p
    } else {
        return;
    };

    if keys.just_pressed(KeyCode::KeyF) {
        commands.spawn(Ripple(p, 0.0));
    }

    if keys.just_pressed(KeyCode::KeyG) {
        let v = random(1200.0, 2000.0);
        let a = random(0.0, 2.0 * std::f32::consts::PI);
        let v = Vec2::from_angle(a) * v;
        commands.spawn(GridParticle(p, v, 0.0));
    }
}

fn spawn_random_ripples(mut commands: Commands, puzzle: Single<&Puzzle>) {
    for (_, v) in puzzle.vertices() {
        if random(0.0, 1.0) < 0.001 {
            commands.spawn(Ripple(v.pos, 0.0));
        }
    }
}

fn update_ripples(
    mut painter: ShapePainter,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Ripple)>,
    time: Res<Time>,
) {
    for (e, mut p) in &mut query {
        p.1 += time.delta_secs();
        let r = p.1 * 60.0;
        let a = (1.0 - p.1) * 0.6;
        if a < 0.0 {
            commands.entity(e).despawn();
        }

        draw_hollow_circle(&mut painter, p.0, -100.0, r, 2.0, GRAY.with_alpha(a));
    }
}

fn update_grid_particles(
    mut painter: ShapePainter,
    mut commands: Commands,
    mut query: Query<(Entity, &mut GridParticle)>,
    time: Res<Time>,
) {
    for (e, mut g) in &mut query {
        g.2 += time.delta_secs();
        if g.2 > 1.0 {
            commands.entity(e).despawn();
        }

        let delta_pos = g.1 * time.delta_secs();
        g.0 += delta_pos;
        g.1 *= 0.96;

        let g = to_grid(g.0);

        fill_grid(&mut painter, g, GREEN.with_alpha(0.3));
    }
}
