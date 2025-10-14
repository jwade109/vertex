use crate::app::VertexApp;
use crate::file_open_system::*;
use crate::puzzle::*;
use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

pub struct EguiEditor;

impl Plugin for EguiEditor {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_systems(EguiPrimaryContextPass, editor_ui_system);
    }
}

fn editor_ui_system(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut app: ResMut<VertexApp>,
    mut puzzle: ResMut<Puzzle>,
    mut sprites: Query<(&Sprite, &Transform)>,
    images: Res<Assets<Image>>,
) {
    egui::Window::new("Editor").show(contexts.ctx_mut().unwrap(), |ui| {
        let x = ui.style_mut();

        x.spacing.item_spacing.y = 10.0;
        x.spacing.button_padding.x = 5.0;
        x.spacing.button_padding.y = 5.0;
        x.visuals.dark_mode = false;
        for x in &mut x.text_styles {
            x.1.size *= 1.5;
        }

        if ui.button("Open Puzzle").clicked() {
            commands.write_message(FileMessage::OpenFile(FileType::Puzzle));
        }

        if ui.button("Open Image").clicked() {
            commands.write_message(FileMessage::OpenFile(FileType::ReferenceImage));
        }

        if ui.button("Complete").clicked() {
            puzzle.complete();
        }

        if ui.button("Decomplete").clicked() {
            puzzle.decomplete();
        }

        if ui.button("Randomize").clicked() {
            puzzle.randomize();
        }

        if ui.button("Triangulate").clicked() {
            puzzle.triangulate();
        }

        if ui.button("Grid").clicked() {
            puzzle.grid();
        }

        if ui.button("Clear Triangles").clicked() {
            puzzle.clear_triangles();
        }

        if ui.button("Sample Colors").clicked() {
            sample_colors(&mut puzzle, sprites, images);
        }

        if ui.button("Clear").clicked() {
            *puzzle = Puzzle::empty();
        }

        if ui.button("Save to File").clicked() {
            println!("Saving to file");
            _ = dbg!(puzzle_to_file(&puzzle, "puzzle.txt"));
        }

        ui.spacing();

        ui.checkbox(&mut app.draw_hidden_edges, "Hidden Edges");
        ui.checkbox(&mut app.puzzle_locked, "Puzzle Locked");
        ui.checkbox(&mut app.draw_missing_edge_indicators, "Edge Indicators");

        ui.add(egui::Slider::new(&mut app.ref_image_alpha, 0.05..=1.0));
        ui.add(egui::Slider::new(&mut app.triangle_alpha, 0.05..=1.0));
    });
}

fn sample_colors(
    puzzle: &mut Puzzle,
    sprites: Query<(&Sprite, &Transform)>,
    images: Res<Assets<Image>>,
) {
    let triangles: Vec<_> = puzzle.triangles().map(|(a, b, c, _)| (a, b, c)).collect();

    for (a, b, c) in triangles {
        let center = (a + b + c) / 3.0;
        for (sprite, tf) in &sprites {
            let img = if let Some(img) = images.get(sprite.image.id()) {
                img
            } else {
                continue;
            };

            let size = img.size().as_ivec2();
            let world_width = size.as_vec2() * tf.scale.xy();

            let sample_color = |q: Vec2| {
                let offset = q - (tf.translation.xy() - world_width / 2.0);
                let offset = offset / world_width * size.as_vec2();
                let offset = offset.as_ivec2();
                if offset.x < 0 || offset.y < 0 || offset.x >= size.x || offset.y >= size.y {
                    None
                } else if let Ok(c) =
                    img.get_color_at(offset.x as u32, size.y as u32 - offset.y as u32 - 1)
                {
                    Some(c.to_srgba())
                } else {
                    None
                }
            };

            let ca = sample_color(a);
            let cb = sample_color(b);
            let cc = sample_color(c);
            let cd = sample_color(center);

            let iter = ca.iter().chain(cb.iter()).chain(cc.iter()).chain(cd.iter());
            let n = iter.clone().count();
            if n == 0 {
                puzzle.set_color(center, PURPLE);
                continue;
            }

            let mut blended = Srgba::BLACK;

            for color in iter {
                blended.red += color.red / n as f32;
                blended.green += color.green / n as f32;
                blended.blue += color.blue / n as f32;
            }

            puzzle.set_color(center, blended);
        }
    }
}
