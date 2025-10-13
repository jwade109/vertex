use crate::math::Vec3;
use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;
use bevy::sprite::Anchor;

#[derive(Clone)]
pub struct TextLabel {
    text: String,
    pos: Vec3,
    #[allow(unused)]
    fontsize: f32,
    height: f32,
    color: Srgba,
    anchor: Anchor,
}

#[derive(Resource)]
pub struct TextPainter {
    pos: Vec3,
    fontsize: f32,
    color: Srgba,
    height: f32,
    anchor: Anchor,
    text: Vec<TextLabel>,
}

impl TextPainter {
    pub fn new() -> Self {
        Self {
            pos: Vec3::ZERO,
            fontsize: 144.0,
            color: BLACK,
            height: 10.0,
            anchor: Anchor::CENTER,
            text: Vec::new(),
        }
    }

    #[allow(unused)]
    pub fn reset(&mut self) {
        self.pos = Vec3::ZERO;
        self.fontsize = 144.0;
        self.color = BLACK;
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.pos = pos;
    }

    pub fn set_color(&mut self, color: Srgba) {
        self.color = color;
    }

    #[allow(unused)]
    pub fn set_font_size(&mut self, fontsize: f32) {
        self.fontsize = fontsize;
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    pub fn text(&mut self, text: impl Into<String>) {
        let label = TextLabel {
            text: text.into(),
            pos: self.pos,
            fontsize: self.fontsize,
            height: self.height,
            color: self.color,
            anchor: self.anchor,
        };
        self.text.push(label);
    }

    fn unpack(&mut self) -> Vec<TextLabel> {
        let ret = self.text.clone();
        self.text.clear();
        ret
    }
}

#[derive(Component)]
pub struct TextLabelComp;

pub fn text_system(
    mut commands: Commands,
    mut tp: ResMut<TextPainter>,
    entities: Query<Entity, With<TextLabelComp>>,
    asset_server: Res<AssetServer>,
) {
    for e in &entities {
        commands.entity(e).despawn();
    }

    let font = asset_server.load("EBGaramond-Medium.ttf");

    for tl in tp.unpack() {
        let label = Text2d::new(tl.text.clone());
        let color = TextColor(tl.color.into());
        let fontsize = TextFont::from_font_size(tl.height).with_font(font.clone());
        let scale = 1.0;
        let transform = Transform::from_scale(Vec3::splat(scale)).with_translation(tl.pos);
        commands.spawn((label, color, transform, fontsize, tl.anchor, TextLabelComp));
    }
}
