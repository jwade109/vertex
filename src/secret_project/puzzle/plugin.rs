use bevy::{time::common_conditions::on_timer, ui::RelativeCursorPosition};

use crate::secret_project::*;

pub struct PuzzlePlugin;

impl Plugin for PuzzlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_cursor_vertex_info,
                get_rel_cursor_info,
                draw_vertices,
                on_puzzle_complete,
                update_title.run_if(in_state(AppState::Playing)),
                draw_vertex_cursor_info.run_if(is_editor_or_playing),
                draw_solution_edges.run_if(is_editor),
                draw_game_edges.run_if(is_menu_or_playing),
                autosave_game_progress
                    .run_if(in_state(AppState::Playing))
                    .run_if(on_timer(std::time::Duration::from_secs(1))),
            ),
        );
        app.add_systems(FixedUpdate, update_confetti);
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

fn autosave_game_progress(
    mut text: MessageWriter<TextMessage>,
    puzzle: Single<Ref<Puzzle>>,
    current: Res<CurrentPuzzle>,
    index: Res<PuzzleIndex>,
) {
    if puzzle.is_changed() {
        let id = match current.0 {
            Some(id) => id,
            _ => return,
        };

        let info = match index.get(&id) {
            Some(info) => info,
            _ => return,
        };

        let path = info.autosave_path();

        info!("Puzzle has been changed since last autosave");
        text.write(TextMessage::debug("Autosaved progress!"));
        if let Err(e) = save_progress(&puzzle, &path) {
            error!("Failed to save: {:?}", e);
            text.write(TextMessage::info("Failed to autosave :("));
        } else {
            info!("Autosaved to {}", path.display());
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct ConfettiVelocity {
    linear: Vec2,
    angular: f32,
}

impl ConfettiVelocity {
    fn random() -> Self {
        Self {
            linear: Vec2::new(random(-160.0, 160.0), random(-160.0, 160.0)),
            angular: random(-14.0, 14.0),
        }
    }
}

fn on_puzzle_complete(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    camera: Single<&Transform, With<Camera>>,
) {
    if !keys.just_pressed(KeyCode::KeyP) {
        return;
    }

    commands.write_message(SoundEffect::UiThreePop);

    for _ in 0..30 {
        let spread = 15.0;
        let p = camera.translation.with_z(0.0)
            + Vec3::new(
                random(-spread, spread),
                random(-spread, spread),
                random(-spread, spread),
            );
        let a = random(0.0, 2.0 * std::f32::consts::PI);
        let l = random(4.0, 20.0);
        let w = random(4.0, 20.0);
        let mesh = Mesh2d(meshes.add(Rectangle::from_size(Vec2::new(l, w))));
        let color = LinearRgba {
            red: rand(),
            green: rand(),
            blue: rand(),
            alpha: 1.0,
        };
        let mat = MeshMaterial2d(materials.add(ColorMaterial::from_color(color)));
        let tf = Transform::from_translation(p).with_rotation(Quat::from_axis_angle(Vec3::Z, a));
        commands.spawn((tf, mesh, mat, ConfettiVelocity::random()));
    }
}

fn update_confetti(
    mut commands: Commands,
    confettis: Query<(Entity, &mut ConfettiVelocity, &mut Transform)>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();
    for (e, mut vel, mut tf) in confettis {
        tf.translation += vel.linear.extend(0.0) * dt;
        tf.rotate_axis(Dir3::Z, vel.angular * dt);

        vel.linear.y -= 90.0 * dt;
        vel.angular *= 0.98;

        tf.scale *= 0.99;

        if tf.scale.x < 0.1 {
            commands.entity(e).despawn();
        }
    }
}
