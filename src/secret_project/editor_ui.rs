use crate::*;

use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

pub struct EguiEditor;

impl Plugin for EguiEditor {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_message::<SavePuzzle>()
            .insert_resource(CurrentPuzzle(None))
            .add_systems(Update, save_puzzle_system)
            .add_systems(EguiPrimaryContextPass, editor_ui_system.run_if(is_editor));
    }
}

#[derive(Message)]
pub struct SavePuzzle;

#[derive(Resource)]
pub struct CurrentPuzzle(pub Option<usize>);

fn save_puzzle_system(
    mut commands: Commands,
    puzzle: Single<&Puzzle>,
    windows: Query<(&RefImagePath, &RefImageWindow)>,
    mut save: MessageReader<SavePuzzle>,
    current: Res<CurrentPuzzle>,
    puzzle_list: Res<PuzzleManifest>,
) {
    if save.is_empty() {
        return;
    }

    for _ in save.read() {}

    let mut images = vec![];

    let id = match current.0 {
        Some(id) => id,
        _ => return,
    };

    let info = match puzzle_list.get(&id) {
        Some(info) => info,
        _ => return,
    };

    info!("Saving puzzle: {:?}", info);

    for (path, window) in windows {
        let path_str = format!("{}", path.0.file_name().unwrap().display());
        let img = ReferenceImage {
            path: path_str,
            pos: window.pos,
        };
        images.push(img);
    }

    match puzzle_to_file(&puzzle, &info.path, images) {
        Ok(()) => {
            commands.write_message(TextMessage::info(format!(
                "Saved puzzle to \"{}\"",
                info.path.display()
            )));
        }
        Err(e) => {
            commands.write_message(TextMessage::info(format!(
                "Failed to save puzzle to \"{}\": {}",
                info.path.display(),
                e
            )));
        }
    }
}

fn editor_ui_system(
    mut contexts: EguiContexts,
    mut commands: Commands,
    mut app: ResMut<Settings>,
    mut puzzle: Single<&mut Puzzle>,
    sprites: Query<(Entity, &Sprite, &Transform)>,
    images: Res<Assets<Image>>,
    keys: Res<ButtonInput<KeyCode>>,
    sel: Res<SelectedVertices>,
    puzzle_list: Res<PuzzleManifest>,
    mut mouse: ResMut<CursorState>,
    camera: Single<&Transform, With<Camera>>,
) {
    mouse.on_egui = false;

    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::KeyS) {
        commands.write_message(SavePuzzle);
    }

    let ctx = contexts.ctx_mut().unwrap();

    egui::Window::new("Editor")
        // .exact_width(300.0)
        .show(ctx, |ui| {
            let x = ui.style_mut();

            x.spacing.item_spacing.y = 10.0;
            x.spacing.button_padding.x = 5.0;
            x.spacing.button_padding.y = 5.0;
            x.visuals.dark_mode = false;
            for x in &mut x.text_styles {
                x.1.size *= 1.5;
            }

            ui.label(format!("{:#?}", *camera));

            ui.collapsing("Puzzles", |ui| {
                for (id, info) in puzzle_list.sorted_list() {
                    if ui.button(&info.title).clicked() {
                        info!("Opening a puzzle: {:?}", info);
                        commands.write_message(OpenPuzzleById(id));
                    }
                }
            });

            ui.collapsing("Color Sampling", |ui| {
                if ui.button("Sample Colors").clicked() {
                    sample_colors(&mut puzzle, &sprites, &images, app.blend_scale);
                }
                ui.add(egui::Slider::new(&mut app.blend_scale, 0.1..=0.9));

                if ui.button("Quantize").clicked() {
                    puzzle.quantize_colors(app.n_colors);
                }
                ui.add(egui::Slider::new(&mut app.n_colors, 3..=500));
            });

            ui.collapsing("Editor", |ui| {
                if ui.button("Open Image").clicked() {
                    commands.write_message(FileMessage::OpenFile(FileType));
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

                if ui.button("Triangulate").clicked() {
                    puzzle.triangulate(sel);
                }

                if ui.button("Update Triangles").clicked() {
                    puzzle.update();
                }

                if ui.button("Clear Triangles").clicked() {
                    puzzle.clear_triangles();
                }

                ui.separator();

                ui.label("Layer Opacity");

                ui.add(egui::Slider::new(&mut app.ref_image_alpha, 0.05..=1.0));
                ui.add(egui::Slider::new(&mut app.triangle_alpha, 0.05..=1.0));
            });

            ui.collapsing("Alerts", |ui| {
                if ui.button("Send Text Alert").clicked() {
                    commands.write_message(TextMessage::info("This is a text alert!"));
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

    mouse.on_egui = ctx.is_pointer_over_area();
}

fn sample_colors(
    puzzle: &mut Puzzle,
    sprites: &Query<(Entity, &Sprite, &Transform)>,
    images: &Res<Assets<Image>>,
    blend_scale: f32,
) {
    let triangles: Vec<_> = puzzle
        .triangles(false)
        .map(|(a, b, c, _)| (a, b, c))
        .collect();

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
                continue;
            }

            let mut blended = Srgba::BLACK;

            for color in iter {
                blended.red += color.red / n as f32;
                blended.green += color.green / n as f32;
                blended.blue += color.blue / n as f32;
            }

            puzzle.set_triangle_color(center, blended);
        }
    }
}
