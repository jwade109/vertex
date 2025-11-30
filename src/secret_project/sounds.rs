use crate::secret_project::*;

use bevy::audio::Volume;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SoundEffect>()
            .add_systems(Update, add_new_sounds);
    }
}

#[derive(Message, Debug)]
pub enum SoundEffect {
    LightPop,
    UiBell,
    UiPopUp,
    UiThreePop,
    UiTrillDown,
    UiTrill,
    Victory,
}

impl SoundEffect {
    fn to_path(&self) -> &'static str {
        match self {
            SoundEffect::LightPop => "sounds/light-pop.mp3",
            SoundEffect::UiBell => "sounds/ui-bell.mp3",
            SoundEffect::UiPopUp => "sounds/ui-pop-up.mp3",
            SoundEffect::UiThreePop => "sounds/ui-three-pop.mp3",
            SoundEffect::UiTrillDown => "sounds/ui-trill-down.mp3",
            SoundEffect::UiTrill => "sounds/ui-trill.mp3",
            SoundEffect::Victory => "sounds/nyt.mp3",
        }
    }

    pub fn all() -> impl Iterator<Item = Self> {
        [
            Self::LightPop,
            Self::UiBell,
            Self::UiPopUp,
            Self::UiThreePop,
            Self::UiTrillDown,
            Self::UiTrill,
        ]
        .into_iter()
    }
}

fn add_new_sounds(
    mut commands: Commands,
    mut sounds: MessageReader<SoundEffect>,
    asset_server: Res<AssetServer>,
) {
    for sound in sounds.read() {
        debug!("{:?}", sound);
        commands.spawn((
            AudioPlayer::new(asset_server.load(sound.to_path())),
            PlaybackSettings::default().with_volume(Volume::Linear(0.2)),
        ));
    }
}
