mod app;
mod button;
mod color_picker;
mod drawing;
mod edge;
mod editor_ui;
mod file_open_system;
mod math;
mod puzzle;
mod reference_image;
mod take_once;
mod text;
mod triangle;
mod vertex;

use crate::app::*;
use crate::button::*;
use crate::color_picker::ColorPicker;
use crate::drawing::*;
use crate::editor_ui::EguiEditor;
use crate::file_open_system::*;
use crate::math::*;
use crate::reference_image::*;
use crate::text::*;
use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::path::PathBuf;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(AssetPlugin {
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            }), // .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(Shape2dPlugin::default())
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_plugins(FilePlugin)
        .add_plugins(ButtonPlugin)
        .add_plugins(ReferenceImagePlugin)
        .add_plugins(EguiEditor)
        .add_systems(Startup, startup)
        .add_systems(FixedUpdate, on_fixed_tick)
        .add_systems(
            Update,
            (on_input_tick, draw_puzzle, text_system, on_load_puzzle).chain(),
        )
        .run();
}

fn startup(mut commands: Commands, mut _windows: Query<&mut Window, With<PrimaryWindow>>) {
    commands.spawn(Camera2d);
    commands.insert_resource(VertexApp::new());
    commands.insert_resource(ClearColor(Srgba::new(0.9, 0.9, 0.9, 1.0).into()));
    commands.insert_resource(TextPainter::new());
    commands.insert_resource(Puzzle::new());

    commands.spawn(ColorPicker::new());

    commands.write_message(FileMessage::Opened(
        FileType::ReferenceImage,
        PathBuf::from("/home/wade/Documents/vertex/potato.jpg"),
    ));

    commands.write_message(FileMessage::Opened(
        FileType::Puzzle,
        PathBuf::from("/home/wade/Documents/vertex/puzzle.txt"),
    ));

    // for mut window in windows.iter_mut() {
    //     // window.set_maximized(true);
    //     window.mode = bevy::window::WindowMode::Fullscreen(
    //         MonitorSelection::Current,
    //         VideoModeSelection::Current,
    //     );
    // }
}

fn on_fixed_tick(mut puzzle: ResMut<Puzzle>) {
    puzzle.step();
}

fn on_load_puzzle(mut puzzle: ResMut<Puzzle>, mut msg: MessageReader<FileMessage>) {
    for msg in msg.read() {
        let path = if let FileMessage::Opened(FileType::Puzzle, path) = msg {
            path
        } else {
            continue;
        };

        if let Ok(p) = puzzle_from_file(&path) {
            *puzzle = p;
        }
    }
}

fn on_input_tick(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut app: ResMut<VertexApp>,
    mut puzzle: ResMut<Puzzle>,
) {
    if let Some(p) = window.cursor_position() {
        let dims = window.size();
        let x = p - dims / 2.0;
        app.mouse_pos = Some(x.with_y(-x.y))
    }

    // keyboard presses
    if keys.just_pressed(KeyCode::KeyQ) {
        if let Some(p) = app.mouse_pos {
            puzzle.add_point(p, true);
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        commands.write_message(AppExit::Success);
    }
}

const TRIANGLE_Z: f32 = 0.09;
const HIDDEN_EDGE_Z: f32 = 0.1;
const ACTIVE_EDGE_Z: f32 = 0.11;
const VERTEX_Z: f32 = 0.2;
const VERTEX_Z_2: f32 = 0.21;
const ACTIVE_LINE_Z: f32 = 0.22;
const CURSOR_Z: f32 = 0.3;

fn draw_puzzle(mut painter: ShapePainter, app: Res<VertexApp>, puzzle: Res<Puzzle>) {
    for (a, b, c, color) in puzzle.triangles() {
        draw_triangle(
            &mut painter,
            a,
            b,
            c,
            TRIANGLE_Z,
            color.with_alpha(app.triangle_alpha),
        );
    }

    let complete = puzzle.is_complete();

    for (a, b, e) in puzzle.edges() {
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
        if !complete && app.draw_hidden_edges {
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

    for v in puzzle.vertices() {
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

        if app.draw_missing_edge_indicators {
            let total_edges = v.invisible_count + v.visible_count;
            for i in 0..v.invisible_count {
                let r = 20.0;
                let a = std::f32::consts::PI * (0.5 + 2.0 * i as f32 / total_edges as f32);
                let p = v.pos + Vec2::from_angle(a) * r;
                draw_circle(&mut painter, p, VERTEX_Z_2, 4.0, BLACK);
            }
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

    draw_cursor_line(&mut painter, &puzzle);
}

fn draw_cursor_line(painter: &mut ShapePainter, puzzle: &Puzzle) -> Option<()> {
    let line = puzzle.active_line()?;
    let start = puzzle.vertex_n(line.0)?;
    draw_line(painter, start.pos, line.1, ACTIVE_LINE_Z, 3.0, ORANGE);
    Some(())
}
