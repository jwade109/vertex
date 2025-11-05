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
        .add_plugins(PuzzleMessagePlugin)
        .add_systems(Startup, startup)
        .add_systems(FixedUpdate, step_puzzle)
        .add_systems(
            Update,
            (
                on_input_tick,
                draw_puzzle,
                draw_cursor_line,
                draw_solution_edges.run_if(not(in_state(EditorMode::Play))),
                draw_game_edges.run_if(in_state(EditorMode::Play)),
                text_system,
                on_load_puzzle,
            ),
        )
        .run();
}

fn startup(mut commands: Commands, mut _windows: Query<&mut Window, With<PrimaryWindow>>) {
    commands.spawn(Camera2d);
    commands.insert_resource(Settings::new());
    commands.insert_resource(ClearColor(Srgba::new(0.9, 0.9, 0.9, 1.0).into()));
    commands.insert_resource(TextPainter::new());

    commands.spawn(Puzzle::new());
    commands.spawn(ColorPicker::new());
}

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
            if let Some(p) = cursor.mouse_pos {
                puzzle.add_point(p);
                commands.write_message(SoundEffect::LightPop);
            }
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        commands.write_message(AppExit::Success);
    }
}
