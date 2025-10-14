mod app;
mod button;
mod color_picker;
mod drawing;
mod edge;
mod lpf;
mod math;
mod puzzle;
mod slider;
mod take_once;
mod text;
mod triangle;
mod ui_element;
mod vertex;
mod window;

use crate::app::*;
use crate::drawing::*;
use crate::math::*;
use crate::take_once::TakeOnce;
use crate::text::*;
use bevy::input::gamepad::{Gamepad, GamepadEvent};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins {})
        .add_plugins(Shape2dPlugin::default())
        .add_systems(Startup, startup)
        .add_systems(FixedUpdate, on_fixed_tick)
        .add_systems(Update, (on_input_tick, on_render_tick, text_system).chain())
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.insert_resource(VertexApp::new());
    commands.insert_resource(ClearColor(Srgba::new(0.9, 0.9, 0.9, 1.0).into()));
    commands.insert_resource(TextPainter::new());
}

fn on_fixed_tick(mut app: ResMut<VertexApp>) {
    app.step()
}

fn on_input_tick(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    gamepad: Query<&Gamepad>,
    mut evr_gamepad: EventReader<GamepadEvent>,
    mut app: ResMut<VertexApp>,
) {
    if let Some(p) = window.cursor_position() {
        let dims = window.size();
        let x = p - dims / 2.0;
        app.mouse_pos = Some(x.with_y(-x.y))
    } else if let Some(g) = gamepad.get_single().ok() {
        let delta = g.left_stick() * 6.0;
        app.mouse_pos = Some(app.mouse_pos.unwrap_or(Vec2::ZERO) + delta);

        if g.just_pressed(GamepadButton::South) {
            app.on_left_mouse_press();
        }

        if g.just_released(GamepadButton::South) {
            app.on_left_mouse_release()
        }

        if g.just_pressed(GamepadButton::East) {
            app.on_right_mouse_press();
        }
    }

    // keyboard presses
    if keys.just_pressed(KeyCode::KeyQ) {
        if let Some(p) = app.mouse_pos {
            app.puzzle.add_point(p, true);
        }
    }

    if keys.just_pressed(KeyCode::KeyC) {
        app.puzzle.complete();
    }

    if keys.just_pressed(KeyCode::KeyR) {
        app.puzzle.randomize();
    }

    if keys.just_pressed(KeyCode::KeyL) {
        app.puzzle = Puzzle::empty();
    }

    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::KeyS) {
        println!("Saving to file");
        _ = dbg!(puzzle_to_file(&app.puzzle, "puzzle.txt"));
    }

    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::KeyO) {
        println!("Opening puzzle file");
        if let Ok(p) = puzzle_from_file("puzzle.txt") {
            app.puzzle = p;
        }
    }

    app.is_snapping = keys.pressed(KeyCode::ShiftLeft);

    // mousebutton presses
    if mouse.just_pressed(MouseButton::Left) {
        app.on_left_mouse_press();
    }

    if mouse.just_released(MouseButton::Left) {
        app.on_left_mouse_release();
    }

    if mouse.just_pressed(MouseButton::Right) {
        app.on_right_mouse_press();
    }

    if mouse.just_released(MouseButton::Right) {
        app.on_right_mouse_release();
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

                app.mouse_pos = Some(app.mouse_pos.unwrap_or(Vec2::ZERO) + delta);
            }
            GamepadEvent::Button(b) => _ = dbg!(b),
            _ => _ = dbg!(e),
        }
    }

    let p = app.mouse_pos;
    app.set_cursor_position(TakeOnce::from_option(p));

    app.ui_elements.sort_by_key(|e| 1 - e.is_clicked() as u8);
}

fn on_render_tick(painter: ShapePainter, app: Res<VertexApp>, mut text: ResMut<TextPainter>) {
    draw_game(painter, &mut text, &app);
}

const TRIANGLE_Z: f32 = 0.09;
const HIDDEN_EDGE_Z: f32 = 0.1;
const ACTIVE_EDGE_Z: f32 = 0.11;
const VERTEX_Z: f32 = 0.2;
const VERTEX_Z_2: f32 = 0.21;
const ACTIVE_LINE_Z: f32 = 0.22;
const CURSOR_Z: f32 = 0.3;

fn draw_game(mut painter: ShapePainter, text: &mut TextPainter, app: &VertexApp) {
    for (a, b, c, color) in app.puzzle.triangles() {
        draw_triangle(&mut painter, a, b, c, TRIANGLE_Z, color);
    }

    let complete = app.puzzle.is_complete();

    for (a, b, e) in app.puzzle.edges() {
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
        if !complete {
            draw_line(
                &mut painter,
                a.pos,
                b.pos,
                HIDDEN_EDGE_Z,
                3.0,
                GRAY.with_alpha(0.2),
            );
        }
    }

    for v in app.puzzle.vertices() {
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

        if v.is_clicked {
            draw_circle(&mut painter, v.pos, VERTEX_Z_2, 8.0, RED);
        }
        if v.is_hovered {
            draw_circle(&mut painter, v.pos, VERTEX_Z_2, 8.0, GREEN);
        }
        if v.is_follow() {
            draw_circle(&mut painter, v.pos, VERTEX_Z_2, 8.0, BLUE);
        }
    }

    if let Some(p) = app.mouse_pos {
        draw_circle(&mut painter, p, CURSOR_Z, 5.0, GRAY.with_alpha(0.3));
    }

    draw_cursor_line(&mut painter, &app.puzzle);

    if app.is_snapping {
        draw_snap_grid(&mut painter, &app.puzzle, app.mouse_pos);
    }

    app.draw(&mut painter, text);
}

fn draw_cursor_line(painter: &mut ShapePainter, puzzle: &Puzzle) -> Option<()> {
    let line = puzzle.active_line()?;
    let start = puzzle.vertex_n(line.0)?;
    draw_line(painter, start.pos, line.1, ACTIVE_LINE_Z, 3.0, ORANGE);
    Some(())
}
