pub mod secret_project;

use secret_project::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            unapproved_path_mode: UnapprovedPathMode::Allow,
            ..default()
        }))
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
        .add_plugins(RevealedTextPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(PuzzlePlugin)
        .add_plugins(AutoSolverPlugin)
        .add_plugins(ConfettiPlugin)
        .add_plugins(NetworkPlugin)
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                on_input_tick,
                draw_cursor_line,
                open_puzzle_by_id,
                update_puzzle_mesh,
                log_app_state_transitions,
                log_in_editor_state_transitions,
                enable_debug_view.run_if(state_changed::<AppState>),
            ),
        )
        .insert_state(AppState::default())
        .add_computed_state::<InEditorOrPlaying>()
        .add_computed_state::<VictoryScreen>()
        // `InputFocus` must be set for accessibility to recognize the button.
        .init_resource::<InputFocus>()
        .run();
}

fn startup(
    mut commands: Commands,
    mut _windows: Query<&mut Window, With<PrimaryWindow>>,
    mut loading: ResMut<NextState<AppState>>,
) {
    commands.spawn(Camera2d);
    commands.insert_resource(Settings::new());
    commands.insert_resource(ClearColor(Srgba::new(0.9, 0.9, 0.9, 1.0).into()));

    commands.spawn(Puzzle::empty("Random"));

    let paths = std::fs::read_dir("./puzzles/").unwrap();
    let mut puzzles = PuzzleIndex::default();

    for (id, path) in paths.enumerate() {
        if let Ok(path) = path {
            let path = path.path();
            let puzzle_file = path.join("puzzle.txt");
            println!("Name: {}", puzzle_file.display());
            if let Ok((puzzle, _)) = puzzle_from_file(puzzle_file.clone()) {
                let info = PuzzleInfo {
                    name: puzzle.title().to_string(),
                    path: puzzle_file,
                };
                puzzles.insert(id, info);
            }
        }
    }

    commands.insert_resource(puzzles);

    loading.set(AppState::Menu);
}

fn enable_debug_view(state: Res<State<AppState>>, mut fps: ResMut<FpsOverlayConfig>) {
    fps.enabled = state.is_editor();
    fps.text_color = Color::BLACK;
    fps.frame_time_graph_config.enabled = state.is_editor();
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
