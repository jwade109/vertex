use crate::secret_project::*;

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_cursor_vertex_info.run_if(is_editor_or_playing),
                get_rel_cursor_info,
                draw_vertices,
                update_title.run_if(is_playing),
                draw_vertex_cursor_info.run_if(camera_is_moveable),
                draw_solution_edges.run_if(is_editor),
                draw_game_edges.run_if(is_menu_or_playing),
                autosave_game_progress
                    .run_if(is_playing)
                    .run_if(on_timer(std::time::Duration::from_secs_f32(0.1))),
                detect_win_condition.run_if(is_playing),
                // experimental animated vertex stuff
                // update_animated_vertices,
                // draw_animated_vertices,
                // nudge_vertices,
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

fn update_cursor_vertex_info(
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
            if a != b {
                commands.write_message(ToggleEdge(a, b));
            }
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

fn draw_game_edges(mut painter: ShapePainter, puzzle: Single<&Puzzle>, save: Res<SaveData>) {
    if puzzle.is_complete(&save) {
        return;
    }
    for (a, b) in puzzle.game_edges(&save) {
        draw_line(&mut painter, a.pos, b.pos, GAME_EDGES_Z, 3.0, BLACK);
    }
}

fn autosave_game_progress(
    mut text: MessageWriter<TextMessage>,
    save: Res<SaveData>,
    current: Res<CurrentPuzzle>,
    manifest: Res<Manifest>,
    install: Res<Installation>,
) {
    if !save.is_changed() {
        return;
    }

    let id = match current.0 {
        Some(id) => id,
        _ => return,
    };

    let info = match manifest.get(id) {
        Some(info) => info,
        _ => return,
    };

    let path = install.save_data_file(&info.short_name);

    info!("Puzzle has been changed since last autosave");

    let save_data: &SaveData = &*save;

    if let Err(e) = save_to_file(save_data, &path) {
        error!("Failed to save: {:?}", e);
        text.write(TextMessage::info("Failed to autosave :("));
    } else {
        info!("Autosaved to {}", path.display());
    }
}

#[allow(unused)]
#[derive(Component)]
pub struct AnimatedVertex {
    index: usize,
    pos: Vec2,
    offset: Vec2,
    velocity: Vec2,
}

#[allow(unused)]
fn update_animated_vertices(
    mut commands: Commands,
    puzzle: Single<&Puzzle>,
    vertices: Query<(Entity, &mut AnimatedVertex)>,
    time: Res<Time<Fixed>>,
) {
    let kp = 10.0;
    let kd = 3.0;

    let dt = time.delta_secs();
    let mut indices = HashSet::new();
    for (e, mut vertex) in vertices {
        if let Some(v) = puzzle.vertex_n(vertex.index) {
            indices.insert(vertex.index);

            let acc = -kp * vertex.offset - kd * vertex.velocity;

            let doff = vertex.velocity * dt;
            vertex.offset += doff;
            vertex.velocity += acc * dt;

            vertex.pos = v.pos;
        } else {
            commands.entity(e).despawn();
        }
    }

    for (id, _) in puzzle.vertices() {
        if !indices.contains(&id) {
            if let Some(v) = puzzle.vertex_n(id) {
                commands.spawn(AnimatedVertex {
                    index: id,
                    pos: v.pos,
                    offset: randvec(10.0, 50.0),
                    velocity: randvec(10.0, 30.0),
                });
            }
        }
    }
}

#[allow(unused)]
fn draw_animated_vertices(
    mut painter: ShapePainter,
    vertices: Query<&AnimatedVertex>,
    camera: Single<&Transform, With<Camera>>,
) {
    for v in vertices {
        painter.reset();
        let p = v.pos + v.offset;
        painter.set_translation(p.extend(50.0));
        painter.set_color(BLACK);
        painter.circle(8.0 * camera.scale.x);
        painter.set_translation(p.extend(51.0));
        painter.set_color(WHITE);
        painter.circle(5.0 * camera.scale.x);
    }
}

#[allow(unused)]
fn nudge_vertices(
    keys: Res<ButtonInput<KeyCode>>,
    puzzle: Single<&Puzzle>,
    mouse: Res<CursorState>,
    vertices: Query<&mut AnimatedVertex>,
) {
    if !keys.pressed(KeyCode::KeyN) {
        return;
    }

    let pos = match mouse.get() {
        Some(p) => p,
        _ => return,
    };

    let (a, b, c) = match puzzle.triangle_at(pos) {
        Some(x) => x,
        _ => return,
    };

    for mut v in vertices {
        if v.index != a && v.index != b && v.index != c {
            continue;
        }

        let delta = (v.pos + v.offset) - pos;
        let u = delta.normalize_or_zero();
        let d = delta.length().max(10.0);
        let mag = 200.0 / d;
        v.velocity += u * mag;
    }
}

fn detect_win_condition(
    puzzle: Single<Ref<Puzzle>>,
    mut save: ResMut<SaveData>,
    mut state: ResMut<NextState<AppState>>,
) {
    if !save.is_changed() {
        return;
    }

    // TODO stopgap
    if puzzle.solution_edges.0.is_empty() {
        info!("Puzzle is empty");
        return;
    }

    if save.is_complete {
        info!("Already complete");
    }

    if !save.is_complete && puzzle.is_complete(&save) {
        save.is_complete = true;
        save.was_ever_complete = true;
        state.set(AppState::Playing { victory: true });
        info!("Victory!");
    }
}
