use crate::*;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorState::default())
            .insert_resource(SelectedVertices::default())
            .insert_state(EditorMode::Edit)
            .add_systems(
                Update,
                (
                    update_cursor_mode,
                    draw_mouse_cursor,
                    draw_selected_vertices.run_if(in_state(EditorMode::Select)),
                    collect_selected_vertices.run_if(in_state(EditorMode::Select)),
                    do_eraser.run_if(in_state(EditorMode::Eraser)),
                    do_brush.run_if(in_state(EditorMode::Brush)),
                    do_select.run_if(in_state(EditorMode::Select)),
                ),
            )
            .add_systems(OnEnter(EditorMode::Select), on_select_enter)
            .add_systems(OnExit(EditorMode::Select), on_select_exit);
    }
}

#[derive(Resource, Default, Debug)]
pub struct CursorState {
    pub mouse_pos: Option<Vec2>,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Sequence)]
pub enum EditorMode {
    Edit,
    Images,
    Select,
    Eraser,
    Brush,
    Play,
}

impl EditorMode {
    pub fn is_play(&self) -> bool {
        *self == EditorMode::Play
    }
}

fn update_cursor_mode(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<EditorMode>>,
    mut next: ResMut<NextState<EditorMode>>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        next.set(next_cycle(&state.get()));
        commands.write_message(TextMessage::new(format!("Switched to {:?}", *next)));
        commands.write_message(SoundEffect::LightPop);
    }
}

pub fn draw_mouse_cursor(
    mut painter: ShapePainter,
    cursor: Res<CursorState>,
    camera: Single<&Transform, With<Camera>>,
) {
    let scale = camera.scale.x;

    if let Some(p) = cursor.mouse_pos {
        fill_circle(&mut painter, p, CURSOR_Z, 5.0 * scale, GRAY.with_alpha(0.3));
    }
}

#[derive(Resource, Default, Deref, DerefMut, Debug)]
pub struct SelectedVertices(pub HashSet<usize>);

pub fn on_select_enter() {
    // TODO
}

pub fn draw_selected_vertices(
    mut painter: ShapePainter,
    puzzle: Single<&Puzzle>,
    v: Res<SelectedVertices>,
    camera: Single<&Transform, With<Camera>>,
) {
    let scale = camera.scale.x;
    for v in &v.0 {
        if let Some(vertex) = puzzle.vertex_n(*v) {
            fill_circle(
                &mut painter,
                vertex.pos,
                SELECTED_VERTEX_Z,
                30.0 * scale,
                BLUE.with_alpha(0.5),
            );
        }
    }
}

pub fn on_select_exit(mut v: ResMut<SelectedVertices>) {
    v.clear();
}

pub fn collect_selected_vertices(
    puzzle: Single<&Puzzle>,
    mut sel: ResMut<SelectedVertices>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if let Some(v) = puzzle.get_clicked_vertex() {
        sel.insert(v);
    }

    if keys.just_pressed(KeyCode::KeyC) {
        sel.0.clear();
    }

    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::KeyA) {
        for v in puzzle.vertices() {
            sel.insert(v.0);
        }
    }
}

fn do_eraser(
    mut painter: ShapePainter,
    cursor: Res<CursorState>,
    camera: Single<&Transform, With<Camera>>,
    lut: Res<SpatialLookup>,
    puzzle: Single<&Puzzle>,
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let scale = camera.scale.x;

    let p = match cursor.mouse_pos {
        Some(p) => p,
        _ => return,
    };

    let is_pressed = buttons.pressed(MouseButton::Left);

    let eraser_world_radius = ERASER_SCREEN_WIDTH * scale;

    draw_circle(&mut painter, p, 100.0, eraser_world_radius, 2.0, RED);

    for g in grids_in_radius(p, eraser_world_radius) {
        for vid in lut.lup_vertex(g).iter().flat_map(|e| e.iter()) {
            if let Some(v) = puzzle.vertex_n(*vid) {
                if p.distance(v.pos) < eraser_world_radius {
                    if is_pressed {
                        commands.write_message(DeleteVertex(*vid));
                    }
                    fill_circle(
                        &mut painter,
                        v.pos,
                        100.0,
                        30.0 * scale,
                        RED.with_alpha(0.5),
                    );
                }
            }
        }

        for (a, b) in lut.lup_edge(g).iter().flat_map(|e| e.iter()) {
            if let Some((v1, v2)) = puzzle.vertex_n(*a).zip(puzzle.vertex_n(*b)) {
                let center = (v1.pos + v2.pos) / 2.0;
                if p.distance(center) < eraser_world_radius {
                    if is_pressed {
                        commands.write_message(DeleteEdge(*a, *b));
                    }
                    fill_circle(
                        &mut painter,
                        center,
                        EDGE_CENTER_HANDLE_Z,
                        30.0 * scale,
                        RED.with_alpha(0.5),
                    );
                }
            }
        }
    }
}

fn do_brush(
    mut painter: ShapePainter,
    cursor: Res<CursorState>,
    camera: Single<&Transform, With<Camera>>,
    lut: Res<SpatialLookup>,
    puzzle: Single<&Puzzle>,
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let scale = camera.scale.x;

    let p = match cursor.mouse_pos {
        Some(p) => p,
        _ => return,
    };

    let is_pressed = buttons.pressed(MouseButton::Left);

    let eraser_world_radius = ERASER_SCREEN_WIDTH * scale;

    draw_circle(&mut painter, p, 100.0, eraser_world_radius, 2.0, GREEN);

    if !is_pressed {
        return;
    }

    let mut count = 0;

    for g in grids_in_radius(p, eraser_world_radius) {
        for vid in lut.lup_vertex(g).iter().flat_map(|e| e.iter()) {
            if let Some(v) = puzzle.vertex_n(*vid) {
                if v.pos.distance(p) < eraser_world_radius {
                    count += 1;
                }
            }
        }
    }

    if count < 5 {
        let r = random(0.3, 0.8) * eraser_world_radius;
        let a = random(0.0, 2.0 * std::f32::consts::PI);
        let q = p + Vec2::from_angle(a) * r;
        commands.write_message(AddVertex(q));
    }
}

fn do_select(
    mut painter: ShapePainter,
    cursor: Res<CursorState>,
    camera: Single<&Transform, With<Camera>>,
    lut: Res<SpatialLookup>,
    mut sel: ResMut<SelectedVertices>,
    puzzle: Single<&Puzzle>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let scale = camera.scale.x;

    let p = match cursor.mouse_pos {
        Some(p) => p,
        _ => return,
    };

    let is_pressed = buttons.pressed(MouseButton::Left);

    let eraser_world_radius = ERASER_SCREEN_WIDTH * scale;

    draw_circle(&mut painter, p, 100.0, eraser_world_radius, 2.0, BLUE);

    if !is_pressed {
        return;
    }

    for g in grids_in_radius(p, eraser_world_radius) {
        for vid in lut.lup_vertex(g).iter().flat_map(|e| e.iter()) {
            if let Some(v) = puzzle.vertex_n(*vid) {
                if v.pos.distance(p) < eraser_world_radius {
                    sel.0.insert(*vid);
                }
            }
        }
    }
}
