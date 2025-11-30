use crate::secret_project::*;

pub struct RevealedTextPlugin;

impl Plugin for RevealedTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, synchronize_to_text);
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
pub struct RevealedText {
    chars: Vec<HiddenChar>,
    timer: Timer,
}

impl RevealedText {
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

    pub fn set_progress(&mut self, progress: f32) {
        info!(progress);
        loop {
            let n = self.chars.len();
            if n == 0 {
                return;
            }
            let n_ideal = (progress * n as f32).round() as usize;
            let n_rev = self.chars.iter().filter(|c| !c.hidden).count();
            if n_rev < n_ideal {
                let i = randint(0, n as i32);
                self.chars[i as usize].hidden = false;
            } else if n_rev > n_ideal {
                let i = randint(0, n as i32);
                self.chars[i as usize].hidden = true;
            } else {
                return;
            }
        }
    }

    pub fn text(&self) -> String {
        self.chars.iter().map(|c| c.char()).collect()
    }
}

// fn update_hidden_text(mut query: Query<&mut RevealedText>, time: Res<Time>) {
//     for mut text in &mut query {
//         text.update(time.delta());
//     }
// }

fn synchronize_to_text(mut query: Query<(&mut Text, &RevealedText)>) {
    for (mut text, ht) in &mut query {
        text.0 = ht.text();
    }
}
