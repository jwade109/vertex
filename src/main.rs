mod edge;
mod gamestate;
mod lpf;
mod math;
mod puzzle;
mod vertex;

use crate::gamestate::*;
use crate::math::*;
use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_vector_shapes::prelude::*;

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
    commands.insert_resource(ClearColor(Srgba::new(0.9, 0.9, 0.9, 1.0).into()))
}

fn on_fixed_tick(mut state: ResMut<GameState>) {
    state.puzzle.step();
}

fn on_input_tick(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut state: ResMut<GameState>,
) {
    let dims = window.size();
    state.mouse_pos = window.cursor_position().map(|p| {
        let x = p - dims / 2.0;
        x.with_y(-x.y)
    });

    if keys.pressed(KeyCode::KeyQ) {
        state
            .puzzle
            .add_point(Vec2::new(random(-1000.0, 1000.0), random(-600.0, 600.0)))
    }

    if keys.just_pressed(KeyCode::KeyR) {
        state.puzzle.randomize();
    }

    if mouse.just_pressed(MouseButton::Left) {
        if let Some(p) = state.mouse_pos {
            state.puzzle.try_toggle_vertex_at(p);
        }
    }

    let x = state.mouse_pos;
    state.puzzle.set_cursor_position(x);
}

fn on_render_tick(painter: ShapePainter, state: Res<GameState>) {
    draw_game(painter, &state);
}

fn draw_circle(painter: &mut ShapePainter, p: Vec2, z: f32, r: f32, color: Srgba) {
    painter.thickness = 3.0;
    painter.hollow = false;
    painter.set_translation(p.extend(z));
    painter.set_color(color);
    painter.circle(r);
    painter.set_translation(Vec3::ZERO);
}

fn draw_triangle(painter: &mut ShapePainter, a: Vec2, b: Vec2, c: Vec2, z: f32, color: Srgba) {
    painter.set_translation(Vec2::ZERO.extend(z));
    painter.set_color(color);
    painter.triangle(a, b, c);
}

fn draw_line(painter: &mut ShapePainter, a: Vec2, b: Vec2, z: f32, thickness: f32, color: Srgba) {
    painter.thickness = thickness;
    painter.set_color(color);
    painter.set_translation(Vec2::ZERO.extend(z));
    painter.line(a.extend(0.0), b.extend(0.0));
}

const TRIANGLE_Z: f32 = 0.0;
const HIDDEN_EDGE_Z: f32 = 0.1;
const ACTIVE_EDGE_Z: f32 = 0.11;
const VERTEX_Z: f32 = 0.2;
const VERTEX_Z_2: f32 = 0.21;

fn draw_game(mut painter: ShapePainter, state: &GameState) {
    for (a, b, c, color) in state.puzzle.triangles() {
        draw_triangle(&mut painter, a, b, c, TRIANGLE_Z, color);
    }

    for (a, b, e) in state.puzzle.edges() {
        let hidden = a.hidden || b.hidden;
        let z = if hidden { HIDDEN_EDGE_Z } else { ACTIVE_EDGE_Z };
        let c = a.pos.lerp(b.pos, 0.5);
        for (v, c) in [(a.pos, c), (b.pos, c)] {
            let r = v.lerp(c, e.portion.actual);
            draw_line(&mut painter, v, r, z, 3.0, BLACK);
        }
    }

    for v in state.puzzle.vertices() {
        let color = if v.hidden { GRAY } else { BLACK };
        draw_circle(&mut painter, v.pos, VERTEX_Z, v.marker_radius.actual, color);
        if !v.hidden {
            draw_circle(
                &mut painter,
                v.pos,
                VERTEX_Z_2,
                v.marker_radius.actual - 4.0,
                WHITE,
            );
        }
    }
}
