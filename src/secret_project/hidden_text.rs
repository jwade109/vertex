use crate::secret_project::*;

pub struct HiddenTextPlugin;

impl Plugin for HiddenTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_hidden_text, synchronize_to_text));
    }
}

#[derive(Debug)]
pub struct HiddenChar {
    c: char,
    hidden: bool,
}

impl HiddenChar {
    fn char(&self) -> char {
        if self.hidden {
            '_'
        } else {
            self.c
        }
    }
}

#[derive(Component, Debug, Default)]
pub struct HiddenText {
    chars: Vec<HiddenChar>,
    timer: Timer,
}

impl HiddenText {
    pub fn new(s: impl Into<String>) -> Self {
        let mut ret = Self::default();
        ret.timer = Timer::from_seconds(0.1, TimerMode::Repeating);
        ret.reset(s);
        ret
    }

    pub fn reset(&mut self, s: impl Into<String>) {
        self.chars = s
            .into()
            .chars()
            .map(|c| HiddenChar { c, hidden: true })
            .collect();
    }

    pub fn update(&mut self, delta: std::time::Duration) {
        self.timer.tick(delta);

        if self.timer.just_finished() {
            for c in &mut self.chars {
                if c.hidden {
                    c.hidden = false;
                    break;
                }
            }
        }
    }

    pub fn text(&self) -> String {
        self.chars.iter().map(|c| c.char()).collect()
    }
}

fn update_hidden_text(mut query: Query<&mut HiddenText>, time: Res<Time>) {
    for mut text in &mut query {
        text.update(time.delta());
    }
}

fn synchronize_to_text(mut query: Query<(&mut Text, &HiddenText)>) {
    for (mut text, ht) in &mut query {
        text.0 = ht.text();
    }
}
