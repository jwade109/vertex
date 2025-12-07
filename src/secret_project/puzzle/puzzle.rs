use crate::secret_project::*;
use kmeans_colors::{get_kmeans, Kmeans};
use palette::Srgb;
use std::collections::HashMap;
use std::path::*;

#[derive(Component)]
pub struct Puzzle {
    title: String,
    next_vertex_id: usize,
    vertices: HashMap<usize, Vertex>,
    pub solution_edges: Edges,
    // pub game_edges: Edges,
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
            triangles: HashMap::new(),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn complete(&mut self, save: &mut SaveData) {
        save.edges = self.solution_edges.clone();
    }

    pub fn decomplete(&mut self, save: &mut SaveData) {
        save.edges.clear();
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

    pub fn vertices(&self) -> impl Iterator<Item = (usize, &Vertex)> + use<'_> {
        self.vertices.iter().map(|(i, v)| (*i, v))
    }

    pub fn vertex_n(&self, n: usize) -> Option<&Vertex> {
        self.vertices.get(&n)
    }

    #[deprecated(note = "This is inefficient. Use the LUT")]
    pub fn vertex_at(&self, p: Vec2, max_radius: f32) -> Option<usize> {
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

    pub fn game_edges<'a>(
        &'a self,
        save: &'a SaveData,
    ) -> impl Iterator<Item = (&'a Vertex, &'a Vertex)> + use<'a> {
        save.edges.0.iter().filter_map(|(a, b)| {
            let v1 = self.vertex_n(*a)?;
            let v2 = self.vertex_n(*b)?;
            Some((v1, v2))
        })
    }

    pub fn triangles<'a>(
        &'a self,
        save: &'a SaveData,
        is_play: bool,
    ) -> impl Iterator<Item = (Vec2, Vec2, Vec2, Srgba)> + use<'a> {
        self.triangles.iter().filter_map(move |((a, b, c), t)| {
            if !self.solution_edges.is_edge(*a, *b)
                || !self.solution_edges.is_edge(*a, *c)
                || !self.solution_edges.is_edge(*b, *c)
            {
                return None;
            }

            if is_play {
                if !save.edges.is_edge(*a, *b)
                    || !save.edges.is_edge(*a, *c)
                    || !save.edges.is_edge(*b, *c)
                {
                    return None;
                }
            }

            let a = self.vertex_n(*a)?.pos;
            let b = self.vertex_n(*b)?.pos;
            let c = self.vertex_n(*c)?.pos;
            Some((a, b, c, t.color))
        })
    }

    pub fn remove_vertex(&mut self, id: usize, save: &mut SaveData) {
        info!("Removing vertex {}", id);
        self.vertices.remove_entry(&id);
        self.solution_edges.remove_vertex(id);
        save.edges.remove_vertex(id);
        self.update_triangles();
    }

    pub fn add_solution_edge(&mut self, a: usize, b: usize) {
        info!("Adding solution edge between {} and {}", a, b);
        self.solution_edges.add_edge(a, b);
        self.update_triangles();
    }

    pub fn add_game_edge(&mut self, a: usize, b: usize, save: &mut SaveData) {
        info!("Adding game edge between {} and {}", a, b);
        save.edges.add_edge(a, b);
    }

    pub fn remove_solution_edge(&mut self, a: usize, b: usize) {
        info!("Adding solution edge between {} and {}", a, b);
        self.solution_edges.remove_edge(a, b);
        self.update_triangles();
    }

    pub fn remove_game_edge(&mut self, a: usize, b: usize) {
        info!("Removing game edge between {} and {}", a, b);
        self.solution_edges.add_edge(a, b);
    }

    pub fn toggle_edge(&mut self, save: &mut SaveData, a: usize, b: usize, is_play: bool) {
        if is_play {
            save.edges.toggle(a, b);
        } else {
            self.solution_edges.toggle(a, b);
            self.update_triangles();
        }
    }

    pub fn triangle_at(&self, p: Vec2) -> Option<(usize, usize, usize)> {
        self.triangles.iter().find_map(|((a, b, c), _)| {
            let pa = self.vertices.get(a)?.pos;
            let pb = self.vertices.get(b)?.pos;
            let pc = self.vertices.get(c)?.pos;
            point_in_triangle(p, pa, pb, pc).then(|| (*a, *b, *c))
        })
    }

    pub fn get_triangle_at(&mut self, p: Vec2) -> Option<&mut Triangle> {
        self.triangles.iter_mut().find_map(|((a, b, c), t)| {
            let a = self.vertices.get(a)?.pos;
            let b = self.vertices.get(b)?.pos;
            let c = self.vertices.get(c)?.pos;
            point_in_triangle(p, a, b, c).then(|| t)
        })
    }

    pub fn set_triangle_color(&mut self, p: Vec2, color: Srgba) {
        if let Some(t) = self.get_triangle_at(p) {
            t.color = color;
        }
    }

    pub fn is_complete(&self, save: &SaveData) -> bool {
        self.solution_edges.0 == save.edges.0
    }

    pub fn progress(&self, save: &SaveData) -> f32 {
        let n_sol = self.triangles(save, false).count();
        let n_game = self.triangles(save, true).count();
        if n_sol == 0 {
            return 0.0;
        }
        n_game as f32 / n_sol as f32
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct PuzzleFileStorage {
    pub title: String,
    pub vertices: HashMap<usize, Vec2>,
    pub edges: Vec<(usize, usize)>,
    pub triangles: Vec<(usize, usize, usize, Srgba)>,
    pub reference_images: Vec<ReferenceImage>,
}

pub fn repr_to_puzzle(value: PuzzleFileStorage) -> (Puzzle, Vec<ReferenceImage>) {
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

fn puzzle_to_repr(value: &Puzzle, images: Vec<ReferenceImage>) -> PuzzleFileStorage {
    let mut repr = PuzzleFileStorage::default();
    repr.title = value.title().to_string();
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
) -> Result<(), VertexError> {
    let repr = puzzle_to_repr(puzzle, images);
    let s = serde_yaml::to_string(&repr)?;
    std::fs::write(filepath, s)?;
    Ok(())
}

pub fn puzzle_from_file(
    filepath: impl Into<PathBuf>,
) -> Result<(Puzzle, Vec<ReferenceImage>), VertexError> {
    let filepath = filepath.into();
    info!("Loading puzzle at {}", filepath.display());
    let repr: PuzzleFileStorage = load_from_file(&filepath)?;
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

pub fn draw_vertices(
    mut painter: ShapePainter,
    puzzle: Single<&Puzzle>,
    save: Single<&SaveData>,
    camera: Single<&Transform, With<Camera>>,
    state: Res<State<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let scale = camera.scale.x;

    let is_play = !state.is_editor();

    if state.is_editor() && keys.pressed(KeyCode::KeyV) {
        return;
    }

    if is_play && puzzle.is_complete(&save) {
        return;
    }

    for (_, v) in puzzle.vertices() {
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
            let color = BLACK;
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
    vinfo: Res<CursorVertexInfo>,
    state: Res<State<AppState>>,
) {
    if let Some(line) = vinfo.active_line {
        if let Some(start) = puzzle.vertex_n(line.0) {
            let color = if !state.is_editor() { BLACK } else { RED };
            draw_line(&mut painter, start.pos, line.1, ACTIVE_LINE_Z, 5.0, color);
        }
    }
}

#[derive(Debug, Clone)]
pub struct PuzzleInstallInfo {
    pub title: String,
    pub short_name: String,
}

impl PuzzleInstallInfo {
    pub fn new(title: String, short_name: String) -> Self {
        Self { title, short_name }
    }
}

#[derive(Resource, Debug, Default, Deref, DerefMut, Clone)]
pub struct PuzzleManifest(HashMap<usize, PuzzleInstallInfo>);

impl PuzzleManifest {
    pub fn sorted_list<'a>(&'a self) -> Vec<(usize, &'a PuzzleInstallInfo)> {
        let mut list: Vec<(usize, &PuzzleInstallInfo)> =
            self.0.iter().map(|(id, info)| (*id, info)).collect();
        list.sort_by_key(|e| e.0);
        list
    }
}

pub fn save_to_file<T: Serialize>(val: &T, path: &Path) -> Result<(), VertexError> {
    let s = serde_yaml::to_string(&val)?;
    Ok(std::fs::write(path, s)?)
}

pub fn load_from_file<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T, VertexError> {
    info!("Loading {}", path.display());
    let s = std::fs::read_to_string(path)?;
    let val: T = serde_yaml::from_str(&s)?;
    Ok(val)
}

pub fn open_puzzle_by_id(
    mut commands: Commands,
    list: Res<PuzzleManifest>,
    install: Res<Installation>,
    all_windows: Query<Entity, With<RefImageWindow>>,
    mut puzzle: Single<&mut Puzzle>,
    mut save_data: Single<&mut SaveData>,
    mut msg: MessageReader<OpenPuzzleById>,
    mut open: ResMut<CurrentPuzzle>,
    mut title: Single<&mut RevealedText, With<UiTitle>>,
    mut number: Query<&mut Text, With<UiNumberLabel>>,
) {
    for msg in msg.read() {
        for e in all_windows {
            commands.entity(e).despawn();
        }
        let id = msg.0;

        let info = match list.get(&id) {
            Some(info) => info,
            _ => continue,
        };

        let path = install.puzzle_file(&info.short_name);

        let (p, images) = match puzzle_from_file(&path) {
            Ok((p, images)) => (p, images),
            Err(e) => {
                let s = format!("{:?}", e);
                commands.write_message(TextMessage::info(s));
                continue;
            }
        };

        **puzzle = p;

        let save_data_path = install.save_data_file(&info.short_name);

        let save = match SaveData::from_file(&save_data_path) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to load save data: {:?}", e);
                SaveData::default()
            }
        };

        **save_data = save;

        for mut number in &mut number {
            number.0 = format!("#{}", id);
        }

        title.reset(puzzle.title());

        commands.write_message(TextMessage::debug(format!(
            "Opened puzzle at \"{}\"",
            path.display()
        )));

        for img in images {
            warn!("Not loading reference image at {}", img.path);
            // let full_path = info.reference_image_path(&img.path);
            // info!("Opening: {:?}, {:?}, {}", img, info, full_path.display());
            // commands.write_message(OpenReferenceImage {
            //     path: full_path,
            //     pos: img.pos,
            // });
        }

        open.0 = Some(id);

        commands.write_message(SoundEffect::UiThreePop);
    }
}

pub fn generate_mesh(puzzle: &Puzzle, save: &SaveData, is_play: bool) -> Option<Mesh> {
    let mut builder = MeshMaker::default();

    for (a, b, c, color) in puzzle.triangles(&save, is_play) {
        builder.set_color(color.into());
        builder.triangle([a, b, c]);
    }

    (!builder.is_empty()).then(|| builder.build())
}

pub fn update_puzzle_mesh(
    mut commands: Commands,
    mut puzzles: Query<(Entity, Ref<Puzzle>, Option<&mut Mesh2d>)>,
    save: Single<Ref<SaveData>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    state: Res<State<AppState>>,
) {
    let is_play = match **state {
        AppState::Menu => true,
        AppState::Playing { .. } => true,
        AppState::Editing { .. } => false,
        _ => return,
    };

    for (e, puzzle, mut mesh_comp) in &mut puzzles {
        if !puzzle.is_changed() && !state.is_changed() && !save.is_changed() {
            continue;
        }

        info!("Mesh update");
        if let Some(mesh) = generate_mesh(&puzzle, &save, is_play) {
            if let Some(m) = &mut mesh_comp {
                **m = Mesh2d(meshes.add(mesh));
            } else {
                let m = Mesh2d(meshes.add(mesh));
                let mat = MeshMaterial2d(materials.add(ColorMaterial::default()));
                let tf = Transform::from_xyz(0.0, 0.0, -100.0);
                commands.entity(e).insert((m, mat, tf, NoFrustumCulling));
            }
        } else {
            info!("Removing mesh");
            commands.entity(e).remove::<Mesh2d>();
        }
    }
}

pub fn update_title(
    puzzles: Query<&Puzzle, Changed<Puzzle>>,
    save: Single<&SaveData>,
    mut text: Single<&mut RevealedText, With<UiTitle>>,
) {
    for puzzle in puzzles {
        let progress = puzzle.progress(&save);
        text.set_progress(progress);
    }
}
