use crate::app::VertexApp;
use crate::file_open_system::*;
use crate::puzzle::*;
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
