use bevy::ui::RelativeCursorPosition;

use crate::secret_project::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UiMessage>();
        app.add_systems(Update, (button_interactions, handle_ui_messages));

        // editor/playing menu
        app.add_systems(OnEnter(InEditorOrPlaying), spawn_playing_menu);
        app.add_systems(OnExit(InEditorOrPlaying), despawn_playing_menu);

        // main menu
        app.add_systems(OnEnter(AppState::Menu), spawn_main_menu);
        app.add_systems(OnExit(AppState::Menu), despawn_main_menu);

        // victory screen
        app.add_systems(OnEnter(VictoryScreen), spawn_victory_screen);
        app.add_systems(OnExit(VictoryScreen), despawn_victory_screen);
    }
}

#[derive(Message, Component, Debug, Clone, Copy)]
pub enum UiMessage {
    Previous,
    Next,
    Save,
    Load,
    Menu,
    Reset,
    Play,
    CloseMenu,
    SetEditorMode(EditorMode),
    Autosolver,
    OpenPuzzle(usize),
    CloseVictoryScreen,
    ExitToDesktop,
}

const BACKGROUND_COLOR: Color = Color::LinearRgba(LinearRgba {
    red: 1.0,
    green: 1.0,
    blue: 1.0,
    alpha: 0.8,
});

const BUTTON_COLOR: Color = Color::LinearRgba(LinearRgba {
    red: 0.6,
    green: 0.6,
    blue: 0.6,
    alpha: 0.9,
});

const HOVER_BUTTON_COLOR: Color = Color::LinearRgba(LinearRgba {
    red: 0.3,
    green: 0.3,
    blue: 0.5,
    alpha: 1.0,
});

const PRESSED_BUTTON_COLOR: Color = Color::LinearRgba(LinearRgba {
    red: 0.15,
    green: 0.15,
    blue: 0.7,
    alpha: 1.0,
});

fn button_interactions(
    mut commands: Commands,
    buttons: Query<(Entity, &Interaction, &UiMessage, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (e, interaction, msg, mut color) in buttons {
        debug!("{}: {:?} {:?}", e, interaction, msg);
        match interaction {
            Interaction::Pressed => {
                commands.write_message(*msg);
                commands.write_message(SoundEffect::LightPop);
                color.0 = PRESSED_BUTTON_COLOR;
            }
            Interaction::Hovered => {
                color.0 = HOVER_BUTTON_COLOR;
            }
            Interaction::None => {
                color.0 = BUTTON_COLOR;
            }
        }
    }
}

// fn on_insert_puzzle_index()

fn header_bar(font: &TextFont) -> impl Bundle {
    let title_labels = (
        Node {
            margin: UiRect::axes(Val::Px(20.0), Val::Px(7.0)),
            min_width: percent(30.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![
            // (
            //     UiNumberLabel,
            //     Node {
            //         margin: UiRect::axes(px(7.0), px(0.0)),
            //         ..default()
            //     },
            //     Text::new(""),
            //     font.clone().with_font_size(30.0),
            //     TextColor(Srgba::gray(0.6).into()),
            //     TextShadow {
            //         offset: Vec2::new(-4.0, 4.0),
            //         color: Srgba::gray(0.2).with_alpha(0.1).into(),
            //     },
            // ),
            (
                UiTitle,
                RevealedText::new(""),
                Text::new(String::new()),
                font.clone().with_font_size(40.0),
                TextColor(BLACK.into()),
                TextShadow {
                    offset: Vec2::new(-4.0, 4.0),
                    color: Srgba::gray(0.2).with_alpha(0.1).into(),
                },
            )
        ],
    );

    (
        PlayingMenuRoot,
        BackgroundColor(BACKGROUND_COLOR),
        BorderColor {
            bottom: Srgba::gray(0.5).into(),
            ..default()
        },
        RelativeCursorPosition::default(),
        Node {
            width: percent(100.0),
            top: Val::ZERO,
            border: UiRect::bottom(Val::Px(3.0)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![
            // make_button("Previous", font, UiMessage::Previous),
            title_labels,
            // make_button("Next", font, UiMessage::Next)
        ],
    )
}

#[derive(Component)]
pub struct UiTitle;

#[derive(Component)]
pub struct UiNumberLabel;

fn footer_bar(commands: &mut Commands, font: &TextFont) {
    // footer bar
    commands
        .spawn((
            PlayingMenuRoot,
            BackgroundColor(BACKGROUND_COLOR),
            BorderColor {
                top: Srgba::gray(0.5).into(),
                ..default()
            },
            RelativeCursorPosition::default(),
            Node {
                width: percent(100.0),
                bottom: Val::ZERO,
                position_type: PositionType::Absolute,
                border: UiRect::top(Val::Px(3.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            let button_names = [
                ("Menu", UiMessage::Menu),
                ("Play", UiMessage::Play),
                ("Reset", UiMessage::Reset),
                ("Edit", UiMessage::SetEditorMode(EditorMode::Edit)),
                ("Images", UiMessage::SetEditorMode(EditorMode::Images)),
                ("Select", UiMessage::SetEditorMode(EditorMode::Select)),
                ("Eraser", UiMessage::SetEditorMode(EditorMode::Eraser)),
                ("Brush", UiMessage::SetEditorMode(EditorMode::Brush)),
                ("Autosolver", UiMessage::Autosolver),
            ];

            for (name, msg) in button_names {
                parent.spawn(make_button(name, font, msg));
            }
        });
}

fn handle_ui_messages(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut messages: MessageReader<UiMessage>,
    mut puzzle: Single<&mut Puzzle>,
    mut solver: ResMut<Autosolver>,
    current: Res<CurrentPuzzle>,
) {
    for msg in messages.read() {
        match msg {
            UiMessage::Previous => {
                state.set(AppState::Playing { victory: false });
                if let Some(id) = current.0 {
                    if id > 0 {
                        commands.write_message(OpenPuzzleById(id - 1));
                    }
                }
            }
            UiMessage::Next => {
                state.set(AppState::Playing { victory: false });
                if let Some(id) = current.0 {
                    commands.write_message(OpenPuzzleById(id + 1));
                }
            }
            UiMessage::Save => (),
            UiMessage::Load => (),
            UiMessage::Reset => {
                puzzle.game_edges.clear();
            }
            UiMessage::Play => state.set(AppState::Playing { victory: false }),
            UiMessage::Menu => state.set(AppState::Menu),
            UiMessage::SetEditorMode(m) => {
                state.set(AppState::Editing { mode: *m });
            }
            UiMessage::Autosolver => {
                solver.toggle();
                commands.write_message(TextMessage::debug("Toggled Autosolver"));
            }
            UiMessage::OpenPuzzle(id) => {
                state.set(AppState::Playing { victory: false });
                commands.write_message(OpenPuzzleById(*id));
            }
            UiMessage::CloseMenu => {
                state.set(AppState::Playing { victory: false });
            }
            UiMessage::CloseVictoryScreen => {
                state.set(AppState::Playing { victory: false });
            }
            UiMessage::ExitToDesktop => {
                commands.write_message(AppExit::Success);
            }
        }
    }
}

fn make_button(s: impl Into<String>, font: &TextFont, msg: UiMessage) -> impl Bundle {
    (
        BackgroundColor(BUTTON_COLOR),
        Node {
            margin: UiRect::all(Val::Px(9.0)),
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(2.0)),
            ..default()
        },
        Button,
        BorderRadius::all(px(4.0)),
        BorderColor::all(GRAY),
        msg,
        children![(
            Text::new(s),
            TextColor(BLACK.into()),
            font.clone().with_font_size(24.0),
            Node {
                margin: UiRect::all(Val::Px(8.0)),
                ..default()
            }
        ),],
    )
}

fn vspace(height: f32) -> impl Bundle {
    (
        Node {
            margin: UiRect::vertical(px(height / 2.0)),
            height: px(2.0),
            width: percent(100.0),
            ..default()
        },
        BackgroundColor(Srgba::gray(0.7).into()),
    )
}

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct PlayingMenuRoot;

fn big_text_node(s: impl Into<String>, font: &TextFont) -> impl Bundle {
    (
        font.clone().with_font_size(60.0),
        Text::new(s),
        TextColor(BLACK.into()),
        Node {
            margin: UiRect::bottom(px(25.0)),
            ..default()
        },
    )
}

fn box_shadow() -> BoxShadow {
    BoxShadow::new(
        BLACK.with_alpha(0.4).into(),
        px(-16.0),
        px(16.0),
        px(0.0),
        px(12.0),
    )
}

fn standard_menu() -> impl Bundle {
    (
        Node {
            // max_width: px(400.0),
            // max_height: px(600.0),
            border: UiRect::all(px(3.0)),
            padding: UiRect::all(px(50.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        box_shadow(),
        BorderRadius::all(px(4.0)),
        RelativeCursorPosition::default(),
        BackgroundColor(BACKGROUND_COLOR),
        BorderColor::all(BLACK),
    )
}

fn main_menu(commands: &mut Commands, font: &TextFont, index: &PuzzleIndex) {
    let header = big_text_node("Secret Project", font);

    let root = commands
        .spawn((
            MenuRoot,
            Node {
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .id();

    let w = commands
        .spawn(standard_menu())
        .with_children(|parent| {
            parent.spawn(header);
            for (id, info) in index.sorted_list() {
                let s = format!("#{}: {}", id, info.name);
                let b = make_button(s, font, UiMessage::OpenPuzzle(id));
                parent.spawn(b);
            }

            parent.spawn(vspace(30.0));

            parent.spawn(make_button(
                "Exit to Desktop",
                font,
                UiMessage::ExitToDesktop,
            ));
        })
        .id();

    commands.entity(root).add_child(w);
}

fn spawn_playing_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning playing menu");
    let font = asset_server.load("EBGaramond-Medium.ttf");
    let font = TextFont::from_font_size(25.0).with_font(font);
    commands.spawn(header_bar(&font));
    footer_bar(&mut commands, &font);
}

fn despawn_playing_menu(mut commands: Commands, query: Query<Entity, With<PlayingMenuRoot>>) {
    info!("Despawning playing menu");
    for e in query {
        commands.entity(e).despawn();
    }
}

fn spawn_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    index: Res<PuzzleIndex>,
) {
    let font = asset_server.load("EBGaramond-Medium.ttf");
    let font = TextFont::from_font_size(25.0).with_font(font);
    main_menu(&mut commands, &font, &index);
}

fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MenuRoot>>) {
    for e in query {
        commands.entity(e).despawn();
    }
}

#[derive(Component)]
struct VictoryScreenRoot;

fn victory_menu(commands: &mut Commands, font: &TextFont) {
    let header = big_text_node("You did it!", font);

    let root = commands
        .spawn((
            VictoryScreenRoot,
            Node {
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .id();

    let w = commands
        .spawn(standard_menu())
        .with_children(|parent| {
            parent.spawn(header);

            let buttons = [
                ("Next Puzzle!", UiMessage::Next),
                ("Stay and Appreciate", UiMessage::CloseVictoryScreen),
                ("Back to Main Menu", UiMessage::Menu),
            ];

            for (s, msg) in buttons {
                parent.spawn(make_button(s, font, msg));
            }
        })
        .id();

    commands.entity(root).add_child(w);
}

fn spawn_victory_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("EBGaramond-Medium.ttf");
    let font = TextFont::from_font_size(25.0).with_font(font);
    victory_menu(&mut commands, &font);
}

fn despawn_victory_screen(mut commands: Commands, query: Query<Entity, With<VictoryScreenRoot>>) {
    for e in query {
        commands.entity(e).despawn();
    }
}
