use crate::secret_project::*;
use bevy::prelude::*;

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

#[derive(Component, Debug)]
pub struct HiddenText {
    chars: Vec<HiddenChar>,
}

impl HiddenText {
    pub fn new(s: impl Into<String>) -> Self {
        Self {
            chars: s
                .into()
                .chars()
                .map(|c| HiddenChar { c, hidden: true })
                .collect(),
        }
    }

    pub fn update(&mut self) {
        for c in &mut self.chars {
            if rand() < 0.05 {
                c.hidden = false;
            }
        }
    }

    pub fn text(&self) -> String {
        self.chars.iter().map(|c| c.char()).collect()
    }
}

fn update_hidden_text(mut query: Query<&mut HiddenText>) {
    for mut text in &mut query {
        text.update();
    }
}

fn synchronize_to_text(mut query: Query<(&mut Text, &HiddenText)>) {
    for (mut text, ht) in &mut query {
        text.0 = ht.text();
    }
}
