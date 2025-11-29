use bevy::ui::RelativeCursorPosition;

use crate::secret_project::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UiMessage>();
        app.add_systems(OnEnter(LoadingState::Done), spawn_ui);
        app.add_systems(Update, (button_interactions, handle_ui_messages));
    }
}

#[derive(Message, Component, Debug, Clone, Copy)]
pub enum UiMessage {
    Previous,
    Next,
    Save,
    Load,
    Reset,
    SetMode(EditorMode),
    Autosolver,
    OpenPuzzle(usize),
}

const BACKGROUND_COLOR: Color = Color::LinearRgba(LinearRgba {
    red: 0.7,
    green: 0.7,
    blue: 0.7,
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
        info!("{}: {:?} {:?}", e, interaction, msg);
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
                HiddenText::new(""),
                Text::new(String::new()),
                font.clone().with_font_size(40.0),
                TextColor(Srgba::BLACK.into()),
                TextShadow {
                    offset: Vec2::new(-4.0, 4.0),
                    color: Srgba::gray(0.2).with_alpha(0.1).into(),
                },
            )
        ],
    );

    (
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
                ("Save", UiMessage::Save),
                ("Load", UiMessage::Load),
                ("Reset", UiMessage::Reset),
                ("Edit", UiMessage::SetMode(EditorMode::Edit)),
                ("Images", UiMessage::SetMode(EditorMode::Images)),
                ("Select", UiMessage::SetMode(EditorMode::Select)),
                ("Eraser", UiMessage::SetMode(EditorMode::Eraser)),
                ("Brush", UiMessage::SetMode(EditorMode::Brush)),
                ("Play", UiMessage::SetMode(EditorMode::Play)),
                ("Autosolver", UiMessage::Autosolver),
            ];

            for (name, msg) in button_names {
                parent.spawn(make_button(name, font, msg));
            }
        });
}

fn handle_ui_messages(
    mut commands: Commands,
    mut state: ResMut<NextState<EditorMode>>,
    mut messages: MessageReader<UiMessage>,
    mut puzzle: Single<&mut Puzzle>,
    mut solver: ResMut<Autosolver>,
    current: Res<CurrentPuzzle>,
) {
    for msg in messages.read() {
        match msg {
            UiMessage::Previous => {
                if let Some(id) = current.0 {
                    if id > 0 {
                        commands.write_message(OpenPuzzleById(id - 1));
                    }
                }
            }
            UiMessage::Next => {
                if let Some(id) = current.0 {
                    commands.write_message(OpenPuzzleById(id + 1));
                }
            }
            UiMessage::Save => (),
            UiMessage::Load => (),
            UiMessage::Reset => {
                puzzle.game_edges.clear();
            }
            UiMessage::SetMode(m) => {
                state.set(*m);
            }
            UiMessage::Autosolver => {
                solver.toggle();
                commands.write_message(TextMessage::debug("Toggled Autosolver"));
            }
            UiMessage::OpenPuzzle(id) => {
                commands.write_message(OpenPuzzleById(*id));
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
        BorderColor::all(GRAY),
        msg,
        children![(
            Text::new(s),
            TextColor(Srgba::BLACK.into()),
            font.clone().with_font_size(24.0),
            TextShadow {
                offset: Vec2::new(-4.0, 4.0),
                color: Srgba::gray(0.2).with_alpha(0.1).into(),
            },
            Node {
                margin: UiRect::all(Val::Px(8.0)),
                ..default()
            }
        ),],
    )
}

fn main_menu(commands: &mut Commands, font: &TextFont, index: &PuzzleIndex) {
    let header = (
        font.clone().with_font_size(60.0),
        Text::new("Secret Project"),
        TextColor(BLACK.into()),
        Node {
            margin: UiRect::bottom(px(25.0)),
            ..default()
        },
    );

    let root = commands
        .spawn(Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();

    let w = commands
        .spawn((
            Node {
                // max_width: px(400.0),
                // max_height: px(600.0),
                border: UiRect::all(px(3.0)),
                padding: UiRect::all(px(50.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            RelativeCursorPosition::default(),
            BackgroundColor(BACKGROUND_COLOR),
            BorderColor::all(BLACK),
        ))
        .with_children(|parent| {
            parent.spawn(header);

            for (id, info) in index.sorted_list() {
                let s = format!("#{}: {}", id, info.name);
                let b = make_button(s, font, UiMessage::OpenPuzzle(id));
                parent.spawn(b);
            }
        })
        .id();

    commands.entity(root).add_child(w);
}

pub fn spawn_ui(mut commands: Commands, asset_server: Res<AssetServer>, index: Res<PuzzleIndex>) {
    let font = asset_server.load("EBGaramond-Medium.ttf");
    let font = TextFont::from_font_size(25.0).with_font(font);
    commands.spawn(header_bar(&font));
    footer_bar(&mut commands, &font);

    main_menu(&mut commands, &font, &index);
}
