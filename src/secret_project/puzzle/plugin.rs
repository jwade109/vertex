use bevy::{time::common_conditions::on_timer, ui::RelativeCursorPosition};

use crate::secret_project::*;

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_vertex_info,
                get_rel_cursor_info,
                draw_vertices,
                draw_vertex_cursor_info.run_if(not(in_state(EditorMode::Play))),
                draw_solution_edges.run_if(not(in_state(EditorMode::Play))),
                draw_game_edges.run_if(in_state(EditorMode::Play)),
                autosave_game_progress
                    .run_if(in_state(EditorMode::Play))
                    .run_if(on_timer(std::time::Duration::from_secs(1))),
            ),
        );
        app.insert_resource(CursorVertexInfo::default());
    }
}

#[derive(Resource, Debug, Default)]
pub struct CursorVertexInfo {
    pub hovered: Option<usize>,
    pub clicked: Option<usize>,
    pub active_line: Option<(usize, Vec2)>,
}

impl CursorVertexInfo {
    fn pair(&self) -> Option<(usize, usize)> {
        self.clicked.zip(self.hovered)
    }
}

fn get_rel_cursor_info(query: Query<&RelativeCursorPosition>, mut cursor: ResMut<CursorState>) {
    cursor.on_ui = false;
    for q in query {
        if q.cursor_over {
            cursor.on_ui = true;
            return;
        }
    }
}

fn update_vertex_info(
    mut commands: Commands,
    mut vinfo: ResMut<CursorVertexInfo>,
    camera: Single<&Transform, With<Camera>>,
    cursor: Res<CursorState>,
    puzzle: Single<&Puzzle>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if let Some(pos) = cursor.get() {
        let r = 50.0 * camera.scale.x;
        vinfo.hovered = puzzle.vertex_at(pos, r);
    } else {
        vinfo.hovered = None;
    }

    if buttons.just_pressed(MouseButton::Left) {
        vinfo.clicked = vinfo.hovered;
    }

    if buttons.just_released(MouseButton::Left) {
        if let Some((a, b)) = vinfo.pair() {
            commands.write_message(ToggleEdge(a, b));
        }
        vinfo.clicked = None;
    }

    if let (Some(clicked), Some(pos)) = (vinfo.clicked, cursor.get()) {
        vinfo.active_line = Some((clicked, pos));
    } else {
        vinfo.active_line = None;
    }
}

fn draw_vertex_cursor_info(
    mut painter: ShapePainter,
    vinfo: Res<CursorVertexInfo>,
    puzzle: Single<&Puzzle>,
    camera: Single<&Transform, With<Camera>>,
) {
    for (id, color) in [(vinfo.hovered, RED), (vinfo.clicked, GREEN)] {
        let id = match id {
            Some(id) => id,
            None => continue,
        };

        let v = match puzzle.vertex_n(id) {
            Some(v) => v,
            None => continue,
        };

        painter.reset();
        painter.set_translation(v.pos.extend(HOVERED_VERTEX_DEBUG_Z));
        painter.set_color(color);
        painter.hollow = true;
        painter.thickness = 3.0;
        painter.thickness_type = ThicknessType::Pixels;
        painter.circle(9.0 * camera.scale.x);
    }
}

pub fn draw_solution_edges(
    mut painter: ShapePainter,
    puzzle: Single<&Puzzle>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.pressed(KeyCode::KeyV) {
        return;
    }
    for (_, a, _, b) in puzzle.solution_edges() {
        draw_line(&mut painter, a.pos, b.pos, SOLUTION_EDGES_Z, 1.0, BLACK);
    }
}

fn draw_game_edges(mut painter: ShapePainter, puzzle: Single<&Puzzle>) {
    if puzzle.is_complete() {
        return;
    }
    for (a, b) in puzzle.game_edges() {
        draw_line(&mut painter, a.pos, b.pos, GAME_EDGES_Z, 3.0, BLACK);
    }
}

fn autosave_game_progress(puzzle: Single<Ref<Puzzle>>) {
    if puzzle.is_changed() {
        info!("Puzzle has been changed since last autosave");
        if let Err(e) = save_progress(&puzzle, Path::new("./save_progress.yaml")) {
            error!("Failed to save: {:?}", e);
        }
    } else {
        info!("No change.");
    }
}
