pub mod secret_project;

use secret_project::*;

use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_dev_tools::fps_overlay::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(AssetPlugin {
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            }), // .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(FpsOverlayPlugin::default())
        .add_plugins(Shape2dPlugin::default())
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_plugins(FilePlugin)
        .add_plugins(ButtonPlugin)
        .add_plugins(ReferenceImagePlugin)
        .add_plugins(EguiEditor)
        .add_plugins(TextAlertPlugin)
        .add_plugins(CameraControllerPlugin)
        .add_plugins(GridPlugin)
        .add_plugins(SoundPlugin)
        .add_plugins(CursorPlugin)
        .add_plugins(ParticlePlugin)
        .add_systems(Startup, startup)
        .add_systems(FixedUpdate, step_puzzle)
        .add_systems(
            Update,
            (on_input_tick, draw_puzzle, text_system, on_load_puzzle).chain(),
        )
        .add_systems(Update, draw_eraser.run_if(in_state(EditorMode::Eraser)))
        .run();
}

fn startup(mut commands: Commands, mut _windows: Query<&mut Window, With<PrimaryWindow>>) {
    commands.spawn(Camera2d);
    commands.insert_resource(VertexApp::new());
    commands.insert_resource(ClearColor(Srgba::new(0.9, 0.9, 0.9, 1.0).into()));
    commands.insert_resource(TextPainter::new());

    commands.spawn(Puzzle::new());
    commands.spawn(ColorPicker::new());
}

fn step_puzzle(mut puzzle: Single<&mut Puzzle>) {
    puzzle.step();
}

fn on_load_puzzle(
    mut commands: Commands,
    mut puzzle: Single<&mut Puzzle>,
    mut msg: MessageReader<FileMessage>,
    mut open: ResMut<OpenPuzzle>,
) {
    for msg in msg.read() {
        let (filetype, path) = if let FileMessage::Opened(filetype, path) = msg {
            (filetype, path)
        } else {
            continue;
        };

        match filetype {
            FileType::Any => (),
            FileType::Puzzle => (),
            FileType::ReferenceImage => continue,
        }

        if let Ok(p) = puzzle_from_file(&path) {
            **puzzle = p;

            commands.write_message(TextMessage::new(format!(
                "Opened puzzle at \"{}\"",
                path.display()
            )));

            open.0 = Some(path.clone());

            commands.write_message(SoundEffect::UiPopUp);
        }
    }
}

fn on_input_tick(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut cursor: ResMut<CursorState>,
    mut puzzle: Single<&mut Puzzle>,
    camera: Single<(&Camera, &GlobalTransform)>,
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
        if let Some(p) = cursor.mouse_pos {
            puzzle.add_point(p, true);
            commands.write_message(SoundEffect::LightPop);
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

const ERASER_SCREEN_WIDTH: f32 = 120.0;

fn draw_eraser(
    mut painter: ShapePainter,
    cursor: Res<CursorState>,
    camera: Single<&Transform, With<Camera>>,
    lut: Res<SpatialLookup>,
    puzzle: Single<&Puzzle>,
) {
    let scale = camera.scale.x;

    let p = match cursor.mouse_pos {
        Some(p) => p,
        _ => return,
    };

    let eraser_world_radius = ERASER_SCREEN_WIDTH * scale;

    draw_circle(
        &mut painter,
        p,
        100.0,
        eraser_world_radius,
        2.0 * scale,
        RED,
    );

    for g in grids_in_radius(p, eraser_world_radius) {
        draw_grid(&mut painter, g, 2.0, GRAY);

        for vid in lut.lup(g).iter().flat_map(|e| e.iter()) {
            if let Some(v) = puzzle.vertex_n(*vid) {
                let color = if p.distance(v.pos) < eraser_world_radius {
                    GREEN
                } else {
                    GREEN.with_alpha(0.2)
                };
                draw_circle(&mut painter, v.pos, 100.0, 25.0, 2.0, color);
            }
        }
    }
}
