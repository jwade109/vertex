use crate::secret_project::*;
use kmeans_colors::{get_kmeans, Kmeans};
use palette::Srgb;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::*;

#[derive(Component)]
pub struct Puzzle {
    title: String,
    next_vertex_id: usize,
    vertices: HashMap<usize, Vertex>,
    solution_edges: Edges,
    game_edges: Edges,
    triangles: HashMap<(usize, usize, usize), Triangle>,
}

fn random_color() -> Srgba {
    let r = rand();
    let g = rand();
    let b = rand();
    Srgba::new(r, g, b, 1.0).mix(&WHITE, 0.2)
}

#[allow(unused)]
fn generate_color_palette(n: usize) -> Vec<Srgba> {
    (0..n).map(|_| random_color()).collect()
}

fn srgba_to_rgb(color: Srgba) -> Srgb {
    Srgb::from_components((color.red, color.green, color.blue))
}

fn rgb_to_srgba(color: Srgb) -> Srgba {
    Srgba::new(color.red, color.green, color.blue, 1.0)
}

impl Puzzle {
    pub fn empty(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            next_vertex_id: 0,
            vertices: HashMap::new(),
            solution_edges: Edges::default(),
            game_edges: Edges::default(),
            triangles: HashMap::new(),
        }
    }

    pub fn new(title: impl Into<String>) -> Self {
        let mut s = Self::empty(title);
        s.randomize();
        s
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn complete(&mut self) {
        self.game_edges.0 = self.solution_edges.0.clone();
    }

    pub fn decomplete(&mut self) {
        self.game_edges.clear();
    }

    pub fn update(&mut self) {
        self.update_triangles();
    }

    pub fn quantize_colors(&mut self, n_colors: u16) {
        let mut indices = Vec::new();
        let mut lab = Vec::new();
        for (k, triangle) in self.triangles.iter() {
            let l = srgba_to_rgb(triangle.color);
            indices.push(*k);
            lab.push(l);
        }

        if lab.is_empty() {
            return;
        }

        let k = n_colors as usize;
        let runs = 10;
        let max_iter = 100;
        let converge = 0.1;
        let verbose = false;
        let seed = 0;

        // Iterate over the runs, keep the best results
        let mut result = Kmeans::new();
        for i in 0..runs {
            let run_result = get_kmeans(k, max_iter, converge, verbose, &lab, seed + i as u64);
            if run_result.score < result.score {
                result = run_result;
            }
        }

        for c in &result.centroids {
            // let color = lab_to_srgba(c);
            dbg!(&c);
        }

        for (i, centroid_id) in result.indices.iter().enumerate() {
            let k = indices[i];
            let triangle = self.triangles.get_mut(&k).unwrap();
            let color = rgb_to_srgba(result.centroids[*centroid_id as usize].clone());
            triangle.color = color;
        }

        // Convert indexed colors back to Srgb<u8> for output
        // let rgb = &result
        //     .centroids
        //     .iter()
        //     .map(|&x| Srgb::from_linear(x.into_color()))
        //     .collect::<Vec<Srgb<u8>>>();
        // let buffer = Srgb::map_indices_to_centroids(&rgb, &result.indices);
    }

    fn update_triangles(&mut self) {
        for (u, v) in &self.solution_edges.0 {
            let u = *u;
            let v = *v;
            for (w, _) in &self.vertices {
                let w = *w;
                if u >= v || v >= w {
                    continue;
                }

                let key = (u, v, w);

                if self.solution_edges.is_edge(v, w) && self.solution_edges.is_edge(u, w) {
                    if self.triangles.contains_key(&key) {
                        continue;
                    }
                    let color = random_color();
                    let t = Triangle::new(color);
                    self.triangles.insert(key, t);
                }
            }
        }

        let mut triangles = self.triangles.clone();
        triangles.retain(|(a, b, c), _| {
            self.solution_edges.is_edge(*a, *b)
                && self.solution_edges.is_edge(*a, *c)
                && self.solution_edges.is_edge(*b, *c)
        });
        self.triangles = triangles;
    }

    fn next_vertex_id(&mut self) -> usize {
        let r = self.next_vertex_id;
        self.next_vertex_id += 1;
        r
    }

    pub fn add_point(&mut self, p: Vec2) {
        let id = self.next_vertex_id();
        self.vertices.insert(id, Vertex::new(p));
        // if let Some((other, pos)) = with_active_edge.then(|| active_line.0).flatten() {
        //     let hovered = self.get_hovered_vertex();

        //     for (_, v) in &mut self.vertices {
        //         v.is_clicked = false;
        //         v.is_hovered = false;
        //     }

        //     let new_id = if let Some(id) = hovered {
        //         if let Some(v) = self.vertices.get_mut(&id) {
        //             v.is_clicked = true;
        //         }
        //         id
        //     } else {
        //         let mut new_vertex = Vertex::new(p);
        //         new_vertex.is_clicked = true;
        //         new_vertex.is_hovered = true;
        //         let id = self.next_vertex_id();
        //         self.vertices.insert(id, new_vertex);
        //         id
        //     };
        //     self.solution_edges.add_edge(new_id, other);
        //     active_line.0 = Some((new_id, pos));
        // } else {
        //     let id = self.next_vertex_id();
        //     self.vertices.insert(id, Vertex::new(p));
        // }

        // self.update();
    }

    pub fn clear_triangles(&mut self) {
        self.solution_edges.clear();
        self.game_edges.clear();
        self.triangles.clear();
        self.update();
    }

    pub fn triangulate(&mut self, sel: Res<SelectedVertices>) {
        let ids: Vec<usize> = sel
            .0
            .iter()
            .filter(|id| self.vertex_n(**id).is_some())
            .map(|id| *id)
            .collect();

        for a in &ids {
            for b in &ids {
                self.solution_edges.remove_edge(*a, *b);
            }
        }

        let points: Vec<_> = ids
            .iter()
            .filter_map(|id| self.vertex_n(*id))
            .map(|v| delaunator::Point {
                x: v.pos.x as f64,
                y: v.pos.y as f64,
            })
            .collect();

        let tri = delaunator::triangulate(&points);

        let mut i = 0;
        loop {
            let slice = match tri.triangles.get(i..i + 3) {
                Some(slice) => slice,
                None => break,
            };

            let a = ids[slice[0]];
            let b = ids[slice[1]];
            let c = ids[slice[2]];

            for (u, v) in [(a, b), (b, c), (a, c)] {
                self.solution_edges.add_edge(u, v);
            }

            i += 3;
        }

        let mut edges = self.solution_edges.0.clone();

        edges.retain(|(a, b)| {
            if let Some((u, v)) = self.vertex_n(*a).zip(self.vertex_n(*b)) {
                u.pos.distance(v.pos) < 100.0
            } else {
                false
            }
        });

        self.solution_edges.0 = edges;

        self.update();
    }

    pub fn randomize(&mut self) {
        self.vertices.clear();
        self.solution_edges.clear();
        self.game_edges.clear();
        self.triangles.clear();
        for _ in 0..30 {
            let v = Vec2::new(random(-700.0, 700.0), random(-400.0, 400.0));
            self.add_point(v);
        }
        self.update();
    }

    pub fn grid(&mut self) {
        self.vertices.clear();
        self.solution_edges.clear();
        self.triangles.clear();

        for x in (-800..=800).step_by(40) {
            for y in (-800..=800).step_by(40) {
                let dx = random(-20.0, 20.0);
                let dy = random(-20.0, 20.0);
                self.add_point(Vec2::new(x as f32 + dx, y as f32 + dy));
            }
        }

        self.update();
    }

    pub fn vertices(&self) -> impl Iterator<Item = (usize, &Vertex)> + use<'_> {
        self.vertices.iter().map(|(i, v)| (*i, v))
    }

    pub fn vertex_n(&self, n: usize) -> Option<&Vertex> {
        self.vertices.get(&n)
    }

    pub fn vertex_at(&self, p: Vec2, max_radius: f32) -> Option<usize> {
        // TODO use lut
        let mut res = None;
        for (i, v) in &self.vertices {
            let d = v.pos.distance(p);
            if d > max_radius {
                continue;
            }
            if let Some((_, dist)) = res {
                if d < dist {
                    res = Some((i, d));
                }
            } else {
                res = Some((i, d));
            }
        }

        res.map(|(i, _)| *i)
    }

    pub fn solution_edges(
        &self,
    ) -> impl Iterator<Item = (usize, &Vertex, usize, &Vertex)> + use<'_> {
        self.solution_edges.0.iter().filter_map(|(a, b)| {
            let v1 = self.vertex_n(*a)?;
            let v2 = self.vertex_n(*b)?;
            Some((*a, v1, *b, v2))
        })
    }

    pub fn game_edges(&self) -> impl Iterator<Item = (&Vertex, &Vertex)> + use<'_> {
        self.game_edges.0.iter().filter_map(|(a, b)| {
            let v1 = self.vertex_n(*a)?;
            let v2 = self.vertex_n(*b)?;
            Some((v1, v2))
        })
    }

    pub fn triangles(&self) -> impl Iterator<Item = (Vec2, Vec2, Vec2, Srgba)> + use<'_> {
        self.triangles.iter().filter_map(|((a, b, c), t)| {
            if !self.solution_edges.is_edge(*a, *b)
                || !self.solution_edges.is_edge(*a, *c)
                || !self.solution_edges.is_edge(*b, *c)
            {
                return None;
            }

            let a = self.vertex_n(*a)?.pos;
            let b = self.vertex_n(*b)?.pos;
            let c = self.vertex_n(*c)?.pos;
            Some((a, b, c, t.color))
        })
    }

    pub fn remove_vertex(&mut self, id: usize) {
        debug!("Removing vertex {}", id);
        self.vertices.remove_entry(&id);
        self.solution_edges.remove_vertex(id);
        self.game_edges.remove_vertex(id);
        self.update_triangles();
    }

    fn on_right_click_down(&mut self, commands: &mut Commands) {
        // TODO get rid of this function.
        if let Some(id) = self.get_hovered_vertex() {
            commands.write_message(DeleteVertex(id));
        }
    }

    fn on_left_click_down(&mut self) {
        for (_, vertex) in &mut self.vertices {
            if vertex.is_hovered {
                vertex.is_clicked = true;
            }
        }
        // self.update();
    }

    fn get_hovered_vertex(&self) -> Option<usize> {
        self.vertices
            .iter()
            .find_map(|(id, v)| v.is_hovered.then(|| *id))
    }

    pub fn get_clicked_vertex(&self) -> Option<usize> {
        self.vertices
            .iter()
            .find_map(|(id, v)| v.is_clicked.then(|| *id))
    }

    pub fn add_solution_edge(&mut self, a: usize, b: usize) {
        info!("Adding solution edge between {} and {}", a, b);
        self.solution_edges.add_edge(a, b);
        self.update_triangles();
    }

    pub fn remove_solution_edge(&mut self, a: usize, b: usize) {
        info!("Adding solution edge between {} and {}", a, b);
        self.solution_edges.remove_edge(a, b);
        self.update_triangles();
    }

    fn on_left_click_up(&mut self) {
        let clicked = self.get_clicked_vertex();
        let hovered = self.get_hovered_vertex();

        if let Some((c, h)) = clicked.zip(hovered) {
            if self.solution_edges.is_edge(c, h) {
                self.remove_solution_edge(c, h);
            } else {
                self.add_solution_edge(c, h)
            }
        }

        for (_, v) in &mut self.vertices {
            v.is_clicked = false;
        }

        // self.update();
    }

    pub fn set_cursor_position(
        &mut self,
        p: &mut TakeOnce<Vec2>,
        scale: f32,
        mut active_line: ResMut<ActiveLine>,
    ) {
        let pos = p.take();
        for (_, v) in &mut self.vertices {
            if !v.is_follow() {
                v.is_hovered = false;
            }
            let base_radius = 8.0;
            let extra = if let Some(pos) = pos {
                let d = pos.distance(v.pos) / scale;
                7.0 * (1.0 - d / 200.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            // v.marker_radius.target = base_radius + extra;
            if v.is_follow() {
                if let Some(p) = pos {
                    v.pos += (p - v.pos) * 0.25;
                }
            }
        }

        if let Some(pos) = pos {
            if let Some(id) = self.vertex_at(pos, CLICK_TARGET_SIZE_PIXELS * scale) {
                if let Some(v) = self.vertices.get_mut(&id) {
                    v.is_hovered = true;
                }
            }
        }

        active_line.0 = || -> Option<(usize, Vec2)> {
            let pos = pos?;
            let idx = self.get_clicked_vertex()?;
            Some((idx, pos))
        }();
    }

    pub fn on_input(&mut self, input: &mut InputMessage, commands: &mut Commands) {
        if input.is_right_pressed() {
            self.on_right_click_down(commands);
            input.dont_propagate();
        } else if input.is_left_pressed() {
            self.on_left_click_down();
            input.dont_propagate();
        } else if input.is_left_released() {
            self.on_left_click_up();
            input.dont_propagate();
        } else if input.is_right_released() {
            // TODO
            // self.on_right_click_down(&mut t);
            // input.dont_propagate();
        }
    }

    fn get_triangle_at(&mut self, p: Vec2) -> Option<&mut Triangle> {
        self.triangles.iter_mut().find_map(|((a, b, c), t)| {
            let a = self.vertices.get(a)?.pos;
            let b = self.vertices.get(b)?.pos;
            let c = self.vertices.get(c)?.pos;
            point_in_triangle(p, a, b, c).then(|| t)
        })
    }

    pub fn set_color(&mut self, p: Vec2, color: Srgba) {
        if let Some(t) = self.get_triangle_at(p) {
            t.color = color;
        }
    }

    pub fn is_complete(&self) -> bool {
        self.solution_edges.0 == self.game_edges.0
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReferenceImage {
    pub path: PathBuf,
    pub pos: Vec2,
}

#[derive(Deserialize, Serialize, Default)]
pub struct PuzzleRepr {
    title: String,
    vertices: HashMap<usize, Vec2>,
    edges: Vec<(usize, usize)>,
    triangles: Vec<(usize, usize, usize, Srgba)>,
    reference_images: Vec<ReferenceImage>,
}

pub fn repr_to_puzzle(value: PuzzleRepr) -> (Puzzle, Vec<ReferenceImage>) {
    let mut puzzle = Puzzle::empty(value.title);
    let mut max_id = 0;
    puzzle.vertices = value
        .vertices
        .into_iter()
        .map(|(id, p)| {
            let v = Vertex::new(p);
            max_id = max_id.max(id);
            (id, v)
        })
        .collect();
    for (a, b) in value.edges {
        puzzle.solution_edges.add_edge(a, b);
    }

    for (a, b, c, color) in value.triangles {
        puzzle.triangles.insert((a, b, c), Triangle::new(color));
    }

    puzzle.next_vertex_id = max_id + 1;

    (puzzle, value.reference_images)
}

fn puzzle_to_repr(value: &Puzzle, images: Vec<ReferenceImage>) -> PuzzleRepr {
    let mut repr = PuzzleRepr::default();
    for (id, p) in &value.vertices {
        repr.vertices.insert(*id, p.pos);
    }
    for (a, b) in &value.solution_edges.0 {
        repr.edges.push((*a, *b));
    }
    for ((a, b, c), t) in &value.triangles {
        repr.triangles.push((*a, *b, *c, t.color));
    }

    repr.reference_images = images;

    repr
}

pub fn puzzle_to_file(
    puzzle: &Puzzle,
    filepath: &Path,
    images: Vec<ReferenceImage>,
) -> Result<(), Box<dyn std::error::Error>> {
    let repr = puzzle_to_repr(puzzle, images);
    let s = serde_yaml::to_string(&repr)?;
    std::fs::write(filepath, s)?;
    Ok(())
}

pub fn puzzle_from_file(
    filepath: impl Into<PathBuf>,
) -> Result<(Puzzle, Vec<ReferenceImage>), Box<dyn std::error::Error>> {
    let filepath = filepath.into();
    let s = std::fs::read_to_string(filepath)?;
    let repr: PuzzleRepr = serde_yaml::from_str(&s)?;
    Ok(repr_to_puzzle(repr))
}

pub fn point_in_triangle(test: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let alpha = ((b.y - c.y) * (test.x - c.x) + (c.x - b.x) * (test.y - c.y))
        / ((b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y));
    let beta = ((c.y - a.y) * (test.x - c.x) + (a.x - c.x) * (test.y - c.y))
        / ((b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y));
    let gamma = 1.0 - alpha - beta;
    alpha > 0.0 && beta > 0.0 && gamma > 0.0
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

pub fn draw_game_edges(mut painter: ShapePainter, puzzle: Single<&Puzzle>) {
    for (a, b) in puzzle.game_edges() {
        draw_line(&mut painter, a.pos, b.pos, GAME_EDGES_Z, 3.0, GRAY);
    }
}

pub fn draw_puzzle(
    mut painter: ShapePainter,
    puzzle: Single<&Puzzle>,
    camera: Single<&Transform, With<Camera>>,
    editor_mode: Res<State<EditorMode>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let scale = camera.scale.x;

    // for (a, b, c, color) in puzzle.triangles() {
    //     draw_triangle(
    //         &mut painter,
    //         a,
    //         b,
    //         c,
    //         TRIANGLE_Z,
    //         color.with_alpha(0.6),
    //     );
    // }

    let is_play = *editor_mode == EditorMode::Play;

    if keys.pressed(KeyCode::KeyV) {
        return;
    }

    for (_, v) in puzzle.vertices() {
        // if v.marker_radius.actual < 1.0 {
        //     continue;
        // }

        let radius = 4.0;

        if is_play {
            fill_circle(&mut painter, v.pos, VERTEX_Z, radius * scale, BLACK);
            fill_circle(
                &mut painter,
                v.pos,
                VERTEX_Z_2,
                (radius - 4.0) * scale,
                WHITE,
            );

            let total_edges = v.invisible_count + v.visible_count;
            for i in 0..total_edges {
                let color = if i < v.invisible_count { BLACK } else { GRAY };
                let r = 20.0 * scale;
                let a = std::f32::consts::PI * (0.5 + 2.0 * i as f32 / total_edges as f32);
                let p = v.pos + Vec2::from_angle(a) * r;
                fill_circle(&mut painter, p, VERTEX_Z_2, 4.0 * scale, color);
            }
        } else {
            let color = if v.is_follow() {
                BLUE
            } else if v.is_clicked {
                RED
            } else if v.is_hovered {
                GREEN
            } else {
                BLACK
            };
            let dims = Vec2::splat(10.0) * scale;
            draw_rect(
                &mut painter,
                v.pos - dims / 2.0,
                dims,
                3.0,
                color,
                GRID_BOUNDS_Z,
            );
        }
    }
}

pub fn draw_cursor_line(
    mut painter: ShapePainter,
    puzzle: Single<&Puzzle>,
    active_line: Res<ActiveLine>,
    state: Res<State<EditorMode>>,
) {
    if let Some(line) = active_line.0 {
        if let Some(start) = puzzle.vertex_n(line.0) {
            let color = if state.is_play() { BLACK } else { RED };
            draw_line(&mut painter, start.pos, line.1, ACTIVE_LINE_Z, 5.0, color);
        }
    }
}

pub fn on_open_puzzle(
    mut commands: Commands,
    all_windows: Query<Entity, With<RefImageWindow>>,
    mut puzzle: Single<&mut Puzzle>,
    mut msg: MessageReader<OpenPuzzle>,
    mut open: ResMut<CurrentPuzzle>,
    mut title: Single<&mut HiddenText, With<UiTitle>>,
) {
    for msg in msg.read() {
        for e in all_windows {
            commands.entity(e).despawn();
        }

        let path = &msg.0;
        match puzzle_from_file(&path) {
            Ok((p, images)) => {
                **puzzle = p;

                title.reset(puzzle.title());

                commands.write_message(TextMessage::new(format!(
                    "Opened puzzle at \"{}\"",
                    path.display()
                )));

                for img in images {
                    println!("Spawn: {:?}", img);
                    commands.write_message(OpenImage(img));
                }

                open.0 = Some(path.clone());

                commands.write_message(SoundEffect::UiPopUp);
            }
            Err(e) => {
                let s = format!("{:?}", e);
                commands.write_message(TextMessage::new(s));
            }
        }
    }
}

pub fn on_load_puzzle(
    mut commands: Commands,
    mut puzzle: Single<&mut Puzzle>,
    mut msg: MessageReader<FileMessage>,
    mut open: ResMut<CurrentPuzzle>,
) {
    for msg in msg.read() {
        let (filetype, path) = if let FileMessage::Opened(filetype, path) = msg {
            (filetype, path)
        } else {
            continue;
        };

        match filetype {
            FileType::Any => (),
            FileType::Puzzle => (),
            FileType::ReferenceImage => continue,
        }

        if let Ok((p, images)) = puzzle_from_file(&path) {
            **puzzle = p;

            commands.write_message(TextMessage::new(format!(
                "Opened puzzle at \"{}\"",
                path.display()
            )));

            for img in images {
                println!("Reference image: {:?}", img);
            }

            open.0 = Some(path.clone());

            commands.write_message(SoundEffect::UiPopUp);
        }
    }
}

pub fn generate_mesh(puzzle: &Puzzle) -> Option<Mesh> {
    let mut builder = MeshMaker::default();

    for (a, b, c, color) in puzzle.triangles() {
        builder.set_color(color.into());
        builder.triangle([a, b, c]);
    }

    (!builder.is_empty()).then(|| builder.build())
}

pub fn update_puzzle_mesh(
    mut commands: Commands,
    mut puzzles: Query<(Entity, &Puzzle, Option<&mut Mesh2d>), Changed<Puzzle>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (e, puzzle, mut mesh_comp) in &mut puzzles {
        info!("Changed!");
        if let Some(mesh) = generate_mesh(puzzle) {
            if let Some(m) = &mut mesh_comp {
                **m = Mesh2d(meshes.add(mesh));
            } else {
                let m = Mesh2d(meshes.add(mesh));
                let mat = MeshMaterial2d(materials.add(ColorMaterial::default()));
                commands.entity(e).insert((m, mat));
            }
        }
    }
}
