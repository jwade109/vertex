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
        .add_systems(Startup, startup.in_set(MainStartup))
        .add_systems(
            Update,
            (
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

    let args: Vec<String> = std::env::args().collect();

    let home_dir = std::env::home_dir().expect("Expected home directory");
    let default = home_dir.join(".vertex_install");

    let install_dir = match args.get(1) {
        Some(s) => PathBuf::from(s),
        None => default,
    };

    let install = Installation::initialize(install_dir).expect("Failed to initialize installation");

    commands.insert_resource(Settings::default());
    commands.insert_resource(install.clone());
    commands.insert_resource(ClearColor(Srgba::new(0.9, 0.9, 0.9, 1.0).into()));

    commands.insert_resource(SaveData::default());
    commands.spawn(Puzzle::default());

    commands.insert_resource(
        Manifest::from_file(&install.network_manifest()).unwrap_or(Manifest::default()),
    );

    commands.write_message(NetworkFetch);

    loading.set(AppState::Loading);
}

fn enable_debug_view(state: Res<State<AppState>>, mut fps: ResMut<FpsOverlayConfig>) {
    fps.enabled = state.is_editor();
    fps.text_color = Color::BLACK;
    fps.frame_time_graph_config.enabled = state.is_editor();
}
