use crate::file_open_system::*;
use bevy::prelude::*;
// use image::RgbaImage;
// use std::path::Path;

pub struct ReferenceImagePlugin;

impl Plugin for ReferenceImagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, insert_new_image);
    }
}

// pub fn read_image(path: &Path) -> Option<RgbaImage> {
//     Some(image::open(path).ok()?.to_rgba8())
// }

// pub fn generate_ship_sprite(vehicle: &Vehicle, parts_dir: &Path, schematic: bool) -> Option<Image> {
//     let dynamic = generate_image(vehicle, parts_dir, schematic)?;
//     let mut img = Image::from_dynamic(
//         dynamic,
//         true,
//         RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
//     );
//     img.sampler = bevy::image::ImageSampler::nearest();
//     Some(img)
// }

fn insert_new_image(mut commands: Commands, mut msg: MessageReader<FileMessage>) {
    for msg in msg.read() {
        let path = if let FileMessage::Opened(FileType::ReferenceImage, path) = msg {
            path
        } else {
            continue;
        };
    }
}
