pub mod secret_project;

use secret_project::*;

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
        .add_plugins(ReferenceImagePlugin)
        .add_plugins(EguiEditor)
        .add_plugins(TextAlertPlugin)
        .add_plugins(CameraControllerPlugin)
        .add_plugins(GridPlugin)
        .add_plugins(SoundPlugin)
        .add_plugins(CursorPlugin)
        .add_plugins(ParticlePlugin)
        .add_plugins(PuzzleMessagePlugin)
        .add_plugins(HiddenTextPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(PuzzlePlugin)
        .add_systems(Startup, startup)
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
                on_open_puzzle,
                update_puzzle_mesh,
                enable_debug_view.run_if(state_changed::<EditorMode>),
            ),
        )
        // `InputFocus` must be set for accessibility to recognize the button.
        .init_resource::<InputFocus>()
        .run();
}

fn startup(mut commands: Commands, mut _windows: Query<&mut Window, With<PrimaryWindow>>) {
    commands.spawn(Camera2d);
    commands.insert_resource(Settings::new());
    commands.insert_resource(ClearColor(Srgba::new(0.9, 0.9, 0.9, 1.0).into()));
    commands.insert_resource(TextPainter::new());

    commands.spawn(Puzzle::new("Random"));

    let paths = std::fs::read_dir("./puzzles/").unwrap();
    let mut puzzles = PuzzleList::default();

    for path in paths {
        if let Ok(path) = path {
            let path = path.path();
            let puzzle_file = path.join("puzzle.txt");
            println!("Name: {}", puzzle_file.display());
            if let Ok((puzzle, _)) = puzzle_from_file(puzzle_file.clone()) {
                puzzles.push((puzzle.title().to_string(), puzzle_file));
            }
        }
    }

    commands.insert_resource(puzzles);
}

#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub struct PuzzleList(Vec<(String, std::path::PathBuf)>);

fn enable_debug_view(state: Res<State<EditorMode>>, mut fps: ResMut<FpsOverlayConfig>) {
    fps.enabled = !state.is_play();
    fps.text_color = Color::BLACK;
    fps.frame_time_graph_config.enabled = !state.is_play();
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
            if let Some(p) = cursor.get() {
                puzzle.add_point(p);
                commands.write_message(SoundEffect::LightPop);
            }
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        commands.write_message(AppExit::Success);
    }
}
