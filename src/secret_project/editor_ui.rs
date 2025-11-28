use crate::*;

use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use egui::containers::panel::Side;
use std::path::PathBuf;

pub struct EguiEditor;

impl Plugin for EguiEditor {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_message::<SavePuzzle>()
            .insert_resource(CurrentPuzzle(None))
            .add_systems(Update, save_puzzle_system)
            .add_systems(
                EguiPrimaryContextPass,
                editor_ui_system.run_if(not(in_state(EditorMode::Play))),
            );
    }
}

#[derive(Message)]
pub struct SavePuzzle {
    filepath: PathBuf,
}

#[derive(Resource)]
pub struct CurrentPuzzle(pub Option<PathBuf>);

fn save_puzzle_system(
    mut commands: Commands,
    puzzle: Single<&Puzzle>,
    windows: Query<(&RefImagePath, &RefImageWindow)>,
    mut save: MessageReader<SavePuzzle>,
) {
    for evt in save.read() {
        let mut images = vec![];

        for (path, window) in windows {
            println!("{}, {}", path.0.display(), window.pos);
            let img = ReferenceImage {
                path: path.0.clone(),
                pos: window.pos,
            };
            images.push(img);
        }

        println!("Saving puzzle to {}", evt.filepath.display());
        match puzzle_to_file(&puzzle, &evt.filepath, images) {
            Ok(()) => {
                commands.write_message(TextMessage::new(format!(
                    "Saved puzzle to \"{}\"",
                    evt.filepath.display()
                )));
            }
            Err(e) => {
                commands.write_message(TextMessage::new(format!(
                    "Failed to save puzzle to \"{}\": {}",
                    evt.filepath.display(),
                    e
                )));
            }
        }
    }
}

fn editor_ui_system(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut app: ResMut<Settings>,
    mut puzzle: Single<&mut Puzzle>,
    mut camera: Single<&mut Transform, (With<Camera>, Without<Sprite>)>,
    sprites: Query<(Entity, &Sprite, &Transform)>,
    images: Res<Assets<Image>>,
    keys: Res<ButtonInput<KeyCode>>,
    open_file: Res<CurrentPuzzle>,
    sel: Res<SelectedVertices>,
    puzzle_list: Res<PuzzleList>,
) {
    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::KeyS) {
        let filepath = open_file.0.clone().unwrap_or("puzzle.txt".into());
        commands.write_message(SavePuzzle { filepath });
    }

    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::KeyO) {
        commands.write_message(FileMessage::OpenFile(FileType::Any));
    }

    egui::SidePanel::new(Side::Right, "Editor")
        .exact_width(300.0)
        .show(contexts.ctx_mut().unwrap(), |ui| {
            let x = ui.style_mut();

            x.spacing.item_spacing.y = 10.0;
            x.spacing.button_padding.x = 5.0;
            x.spacing.button_padding.y = 5.0;
            x.visuals.dark_mode = false;
            for x in &mut x.text_styles {
                x.1.size *= 1.5;
            }

            ui.collapsing("Camera", |ui| {
                let mut scale = camera.scale.x;

                ui.add(egui::Slider::new(
                    &mut camera.translation.x,
                    -50000.0..=50000.0,
                ));
                ui.add(egui::Slider::new(
                    &mut camera.translation.y,
                    -50000.0..=50000.0,
                ));
                ui.add(egui::Slider::new(
                    &mut camera.translation.z,
                    -5000.0..=5000.0,
                ));
                ui.add(egui::Slider::new(&mut scale, 0.01..=10.0));

                camera.scale.x = scale;
                camera.scale.y = scale;
            });

            ui.collapsing("Puzzles", |ui| {
                for puzzle in puzzle_list.iter() {
                    if ui.button(format!("{}", puzzle.display())).clicked() {
                        commands.write_message(OpenPuzzle(puzzle.clone()));
                    }
                }
            });

            ui.collapsing("Editor", |ui| {
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

                if ui.button("Update").clicked() {
                    puzzle.update();
                }

                if ui.button("Randomize").clicked() {
                    puzzle.randomize();
                }

                if ui.button("Triangulate").clicked() {
                    puzzle.triangulate(sel);
                }

                if ui.button("Update Triangles").clicked() {
                    puzzle.update();
                }

                if ui.button("Grid").clicked() {
                    puzzle.grid();
                }

                if ui.button("Clear Triangles").clicked() {
                    puzzle.clear_triangles();
                }

                if ui.button("Clear").clicked() {
                    **puzzle = Puzzle::empty("Empty");
                    for (e, ..) in &sprites {
                        commands.entity(e).despawn();
                    }
                }

                if ui.button("Save to File").clicked() {
                    commands.write_message(SavePuzzle {
                        filepath: "puzzle.txt".into(),
                    });
                }

                ui.separator();

                ui.label("Color Sampling");

                if ui.button("Sample Colors").clicked() {
                    sample_colors(&mut puzzle, &sprites, &images, app.blend_scale);
                }
                ui.add(egui::Slider::new(&mut app.blend_scale, 0.1..=0.9));

                if ui.button("Quantize").clicked() {
                    puzzle.quantize_colors(app.n_colors);
                }
                ui.add(egui::Slider::new(&mut app.n_colors, 3..=500));

                ui.separator();

                ui.label("Layer Opacity");

                ui.add(egui::Slider::new(&mut app.ref_image_alpha, 0.05..=1.0));
                ui.add(egui::Slider::new(&mut app.triangle_alpha, 0.05..=1.0));
            });

            ui.collapsing("Alerts", |ui| {
                if ui.button("Send Text Alert").clicked() {
                    commands.write_message(TextMessage::new("This is a text alert!"));
                }
            });

            ui.collapsing("Sounds", |ui| {
                for sfx in SoundEffect::all() {
                    if ui.button(format!("{:?}", sfx)).clicked() {
                        commands.write_message(sfx);
                    }
                }
            });
        });
}

fn sample_colors(
    puzzle: &mut Puzzle,
    sprites: &Query<(Entity, &Sprite, &Transform)>,
    images: &Res<Assets<Image>>,
    blend_scale: f32,
) {
    let triangles: Vec<_> = puzzle.triangles().map(|(a, b, c, _)| (a, b, c)).collect();

    for (a, b, c) in triangles {
        let center = (a + b + c) / 3.0;
        let a = center.lerp(a, blend_scale);
        let b = center.lerp(b, blend_scale);
        let c = center.lerp(c, blend_scale);
        for (_, sprite, tf) in sprites {
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
                // puzzle.set_color(center, Srgba::NONE);
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
