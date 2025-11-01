use bevy::prelude::*;

pub struct TextAlertPlugin;

impl Plugin for TextAlertPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TextMessage>()
            .add_systems(Update, (update_text_alerts, process_messages));
    }
}

#[derive(Message, Debug, Clone)]
pub struct TextMessage(String);

#[derive(Component, Debug, Clone)]
pub struct TextAlert {
    age: f32,
}

impl TextMessage {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

fn process_messages(
    mut commands: Commands,
    mut msg: MessageReader<TextMessage>,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("EBGaramond-Medium.ttf");
    for msg in msg.read() {
        info!("{}", &msg.0);

        let t = Text::new(msg.0.clone());
        let f = TextFont::from_font_size(55.0).with_font(font.clone());
        let c = TextColor::BLACK;
        let s = TextShadow {
            offset: Vec2::new(-4.0, 4.0),
            color: Srgba::gray(0.2).with_alpha(0.3).into(),
        };

        let border_width = 0.0;

        let b = BorderColor::all(Srgba::RED);


        commands
            .spawn((
                Node {
                    bottom: Val::ZERO,
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    overflow: Overflow::visible(),
                    width: Val::Vw(100.0),
                    height: Val::Vh(70.0),
                    border: UiRect::all(Val::Px(border_width)),
                    ..default()
                },
                b,
            ))
            .with_children(|builder| {
                builder.spawn((b, t, f, c, s, TextAlert { age: 0.0 }));
            });
    }
}

fn update_text_alerts(
    mut commands: Commands,
    mut text: Query<(&mut Node, &mut TextAlert, &mut TextColor, &ChildOf)>,
    time: Res<Time>,
) {
    for (mut node, mut ta, mut color, child_of) in &mut text {
        node.top = Val::Percent(-ta.age * 6.0);
        ta.age += time.delta_secs();
        let a = 1.0 - ta.age;
        if a < 0.0 {
            commands.entity(child_of.0).despawn();
            continue;
        }
        let a = a.sqrt();
        color.0.set_alpha(a);
    }
}
