mod edge;
mod gamestate;
mod lpf;
mod math;
mod puzzle;
mod triangle;
mod vertex;

use crate::gamestate::*;
use crate::math::*;
use crate::puzzle::*;
use bevy::color::palettes::css::*;
use bevy::input::gamepad::{Gamepad, GamepadEvent};
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
    gamepad: Query<&Gamepad>,
    mut evr_gamepad: EventReader<GamepadEvent>,
    mut state: ResMut<GameState>,
) {
    if let Some(p) = window.cursor_position() {
        let dims = window.size();
        let x = p - dims / 2.0;
        state.mouse_pos = Some(x.with_y(-x.y))
    } else if let Some(g) = gamepad.get_single().ok() {
        let delta = g.left_stick() * 6.0;
        state.mouse_pos = Some(state.mouse_pos.unwrap_or(Vec2::ZERO) + delta);

        if g.just_pressed(GamepadButton::South) {
            if let Some(p) = state.mouse_pos {
                state.puzzle.on_left_click_down(p);
            }
        }

        if g.just_released(GamepadButton::South) {
            state.puzzle.on_left_click_up();
        }

        if g.just_pressed(GamepadButton::East) {
            if let Some(p) = state.mouse_pos {
                state.puzzle.on_right_click_down(p);
            }
        }
    }

    // keyboard presses
    if keys.just_pressed(KeyCode::KeyQ) {
        if let Some(p) = state.mouse_pos {
            state.puzzle.add_point(p);
        }
    }

    if keys.just_pressed(KeyCode::KeyR) {
        state.puzzle.randomize();
    }

    // mousebutton presses
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(p) = state.mouse_pos {
            state.puzzle.on_left_click_down(p);
        }
    }

    if mouse.just_released(MouseButton::Left) {
        state.puzzle.on_left_click_up();
    }

    if mouse.just_pressed(MouseButton::Right) {
        if let Some(p) = state.mouse_pos {
            state.puzzle.on_right_click_down(p);
        }
    }

    // gamepad events
    for e in evr_gamepad.read() {
        match e {
            GamepadEvent::Axis(axis) => {
                let delta = match axis.axis {
                    GamepadAxis::LeftStickX => Vec2::X * axis.value,
                    GamepadAxis::LeftStickY => Vec2::Y * axis.value,
                    _ => continue,
                };

                dbg!(delta);

                state.mouse_pos = Some(state.mouse_pos.unwrap_or(Vec2::ZERO) + delta);
            }
            GamepadEvent::Button(b) => _ = dbg!(b),
            _ => _ = dbg!(e),
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
const ACTIVE_LINE_Z: f32 = 0.22;
const CURSOR_Z: f32 = 0.3;

fn draw_game(mut painter: ShapePainter, state: &GameState) {
    for (a, b, c, color) in state.puzzle.triangles() {
        draw_triangle(&mut painter, a, b, c, TRIANGLE_Z, color);
    }

    for (a, b, e) in state.puzzle.edges() {
        let z = if e.is_visible {
            ACTIVE_EDGE_Z
        } else {
            HIDDEN_EDGE_Z
        };
        let c = a.pos.lerp(b.pos, 0.5);
        for (v, c) in [(a.pos, c), (b.pos, c)] {
            let r = v.lerp(c, e.length_animation.actual);
            draw_line(&mut painter, v, r, z, e.thickness_animation.actual, BLACK);
        }
        draw_line(
            &mut painter,
            a.pos,
            b.pos,
            HIDDEN_EDGE_Z,
            3.0,
            GRAY.with_alpha(0.2),
        );
    }

    for v in state.puzzle.vertices() {
        if v.marker_radius.actual < 1.0 {
            continue;
        }

        draw_circle(&mut painter, v.pos, VERTEX_Z, v.marker_radius.actual, BLACK);
        draw_circle(
            &mut painter,
            v.pos,
            VERTEX_Z_2,
            v.marker_radius.actual - 4.0,
            WHITE,
        );

        let total_edges = v.invisible_count + v.visible_count;
        for i in 0..v.invisible_count {
            let r = 20.0;
            let a = std::f32::consts::PI * (0.5 + 2.0 * i as f32 / total_edges as f32);
            let p = v.pos + Vec2::from_angle(a) * r;
            draw_circle(&mut painter, p, VERTEX_Z_2, 4.0, BLACK);
        }
    }

    if let Some(p) = state.mouse_pos {
        draw_circle(&mut painter, p, CURSOR_Z, 5.0, GRAY.with_alpha(0.3));
    }

    draw_cursor_line(painter, &state.puzzle);
}

fn draw_cursor_line(mut painter: ShapePainter, puzzle: &Puzzle) -> Option<()> {
    let line = puzzle.active_line()?;
    let start = puzzle.vertex_n(line.0)?;
    draw_line(&mut painter, start.pos, line.1, ACTIVE_LINE_Z, 3.0, ORANGE);
    Some(())
}
