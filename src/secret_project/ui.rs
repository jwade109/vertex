use crate::secret_project::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui);
        app.add_systems(Update, button_interactions);
    }
}

#[derive(Message, Component, Debug)]
pub enum UiMessage {
    Previous,
    Next,
    Save,
    Load,
    Reset,
}

fn button_interactions(
    mut commands: Commands,
    buttons: Query<(Entity, &Interaction, &UiMessage), Changed<Interaction>>,
) {
    for (e, interaction, msg) in buttons {
        let s = format!("{}: {:?} {:?}", e, interaction, msg);
        commands.write_message(TextMessage::new(s));
    }
}

fn header_bar(commands: &mut Commands, font: &TextFont) {
    // header bar
    commands
        .spawn((
            BackgroundColor(Srgba::WHITE.with_alpha(0.5).into()),
            BorderColor {
                bottom: Srgba::gray(0.5).into(),
                ..default()
            },
            Node {
                width: percent(100.0),
                top: Val::ZERO,
                border: UiRect::bottom(Val::Px(3.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_child(make_button("Previous", font, UiMessage::Previous))
        .with_child((
            HiddenText::new("Very Long Puzzle Title"),
            Text::new(String::new()),
            font.clone().with_font_size(40.0),
            TextColor(Srgba::BLACK.into()),
            Node {
                margin: UiRect::all(Val::Px(7.0)),
                ..default()
            },
            TextShadow {
                offset: Vec2::new(-4.0, 4.0),
                color: Srgba::gray(0.2).with_alpha(0.1).into(),
            },
            UiTitle,
        ))
        .with_child(make_button("Next", font, UiMessage::Next));
}

#[derive(Component)]
pub struct UiTitle;

fn footer_bar(commands: &mut Commands, font: &TextFont) {
    // footer bar
    commands
        .spawn((
            BackgroundColor(Srgba::WHITE.with_alpha(0.3).into()),
            BorderColor {
                top: Srgba::gray(0.5).into(),
                ..default()
            },
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
            ];

            for (name, msg) in button_names {
                parent.spawn(make_button(name, font, msg));
            }
        });
}

fn make_button(s: impl Into<String>, font: &TextFont, msg: UiMessage) -> impl Bundle {
    (
        BackgroundColor(Srgba::gray(0.8).into()),
        Node {
            margin: UiRect::all(Val::Px(9.0)),
            ..default()
        },
        Button,
        msg,
        children![(
            HiddenText::new(s.into()),
            Text::new(String::new()),
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

pub fn spawn_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("EBGaramond-Medium.ttf");
    let font = TextFont::from_font_size(25.0).with_font(font);
    header_bar(&mut commands, &font);
    footer_bar(&mut commands, &font);
}
