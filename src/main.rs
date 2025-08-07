use crate::gamestate::GameState;
use crate::math::*;
use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

mod gamestate;
mod math;
mod puzzle;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins {})
        .add_plugins(Shape2dPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(FixedUpdate, on_fixed_tick)
        .add_systems(Update, (on_input_tick, on_render_tick).chain())
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.insert_resource(GameState::new());
}

fn on_fixed_tick() {}

fn on_input_tick(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<GameState>) {
    if keys.pressed(KeyCode::KeyQ) {
        state
            .puzzle
            .add_point(Vec2::new(rand() * 400.0, rand() * 400.0))
    }

    if keys.just_pressed(KeyCode::KeyR) {
        state.puzzle.randomize();
    }
}

fn on_render_tick(painter: ShapePainter, state: Res<GameState>) {
    draw_game(painter, &state);
}

pub fn draw_circle(gizmos: &mut Gizmos, p: Vec2, r: f32, color: Srgba) {
    let iso = Isometry2d::from_translation(p);
    gizmos.circle_2d(iso, r, color);
}

fn draw_game(mut painter: ShapePainter, state: &GameState) {
    painter.thickness = 3.0;
    // painter.hollow = true;
    painter.cap = Cap::Round;

    for p in state.puzzle.vertices() {
        painter.set_translation(p.extend(0.0));
        painter.circle(6.0);
    }

    // painter.hollow = false;
    painter.set_translation(Vec3::ZERO);

    for (p1, p2, p3, color) in state.puzzle.triangles() {
        painter.color = color.with_alpha(0.1).into();
        painter.triangle(p1, p2, p3);
    }

    painter.color = WHITE.into();

    for (p1, p2, p3, _) in state.puzzle.triangles() {
        painter.line(p1.extend(0.0), p2.extend(0.0));
        painter.line(p1.extend(0.0), p3.extend(0.0));
        painter.line(p2.extend(0.0), p3.extend(0.0));
    }
}
