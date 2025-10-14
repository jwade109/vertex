use crate::edge::*;
use crate::math::*;
use crate::take_once::*;
use crate::triangle::*;
use crate::vertex::*;
use bevy::color::palettes::basic::*;
use bevy::prelude::*;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const CLICK_TARGET_SIZE_PIXELS: f32 = 50.0;

#[derive(Resource)]
pub struct Puzzle {
    palette: Vec<Srgba>,
    next_vertex_id: usize,
    vertices: HashMap<usize, Vertex>,
    edges: HashMap<(usize, usize), Edge>,
    triangles: HashMap<(usize, usize, usize), Triangle>,
    active_line: Option<(usize, Vec2)>,
}

fn generate_color_palette(n: usize) -> Vec<Srgba> {
    (0..n)
        .map(|_| {
            let r = rand();
            let g = rand();
            let b = rand();
            Srgba::new(r, g, b, 1.0).mix(&WHITE, 0.2)
        })
        .collect()
}

impl Puzzle {
    pub fn empty() -> Self {
        Self {
            palette: generate_color_palette(8),
            next_vertex_id: 0,
            vertices: HashMap::new(),
            edges: HashMap::new(),
            triangles: HashMap::new(),
            active_line: None,
        }
    }

    pub fn new() -> Self {
        let mut s = Self::empty();
        s.randomize();
        s
    }

    pub fn complete(&mut self) {
        for (_, e) in &mut self.edges {
            e.is_visible = true;
        }
    }

    pub fn decomplete(&mut self) {
        for (_, e) in &mut self.edges {
            e.is_visible = false;
        }
    }

    pub fn update_triangles(&mut self) {
        for (_, edge) in &self.edges {
            let u = edge.a;
            let v = edge.b;
            for (w, _) in &self.vertices {
                let w = *w;
                if u >= v || v >= w {
                    continue;
                }

                let key = (u, v, w);

                if self.is_edge(v, w) && self.is_edge(u, w) {
                    if self.triangles.contains_key(&key) {
                        continue;
                    }
                    let color = self.palette.choose(&mut rand::rng()).unwrap();
                    let t = Triangle::new(u, v, w, *color);
                    self.triangles.insert(key, t);
                }
            }
        }

        let mut triangles = self.triangles.clone();
        triangles.retain(|_, t| {
            self.is_edge(t.a, t.b) && self.is_edge(t.a, t.c) && self.is_edge(t.b, t.c)
        });
        self.triangles = triangles;
    }

    fn next_vertex_id(&mut self) -> usize {
        let r = self.next_vertex_id;
        self.next_vertex_id += 1;
        r
    }

    pub fn add_point(&mut self, p: Vec2, with_active_edge: bool) {
        if let Some((other, pos)) = with_active_edge.then(|| self.active_line).flatten() {
            let hovered = self.get_hovered_vertex();

            for (_, v) in &mut self.vertices {
                v.is_clicked = false;
                v.is_hovered = false;
            }

            let new_id = if let Some(id) = hovered {
                if let Some(v) = self.vertices.get_mut(&id) {
                    v.is_clicked = true;
                }
                id
            } else {
                let mut new_vertex = Vertex::new(p);
                new_vertex.is_clicked = true;
                new_vertex.is_hovered = true;
                let id = self.next_vertex_id();
                self.vertices.insert(id, new_vertex);
                id
            };
            self.add_edge(new_id, other, false);
            self.active_line = Some((new_id, pos));
        } else {
            let id = self.next_vertex_id();
            self.vertices.insert(id, Vertex::new(p));
        }
    }

    pub fn randomize(&mut self) {
        self.palette = generate_color_palette(8);
        self.vertices.clear();
        self.edges.clear();
        self.triangles.clear();
        for _ in 0..70 {
            let v = Vec2::new(random(-700.0, 700.0), random(-400.0, 400.0));
            self.add_point(v, false);
        }

        let ids: Vec<_> = self.vertices.iter().map(|(id, _)| *id).collect();

        let points: Vec<_> = self
            .vertices
            .iter()
            .map(|(_, v)| delaunator::Point {
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
                let min = u.min(v);
                let max = u.max(v);
                let edge = Edge::new(min, max, true);

                self.edges.insert((min, max), edge);
            }

            i += 3;
        }

        self.update_triangles();
    }

    pub fn vertices(&self) -> impl Iterator<Item = &Vertex> + use<'_> {
        self.vertices.iter().map(|(_, v)| v)
    }

    pub fn vertex_n(&self, n: usize) -> Option<&Vertex> {
        self.vertices.get(&n)
    }

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

    pub fn edges(&self) -> impl Iterator<Item = (&Vertex, &Vertex, &Edge)> + use<'_> {
        self.edges.iter().filter_map(|(_, e)| {
            let v1 = self.vertex_n(e.a)?;
            let v2 = self.vertex_n(e.b)?;
            Some((v1, v2, e))
        })
    }

    pub fn triangles(&self) -> impl Iterator<Item = (Vec2, Vec2, Vec2, Srgba)> + use<'_> {
        self.triangles.iter().filter_map(|(_, t)| {
            if t.animation.actual < 0.01 {
                return None;
            }

            let a = self.vertex_n(t.a)?.pos;
            let b = self.vertex_n(t.b)?.pos;
            let c = self.vertex_n(t.c)?.pos;

            let s = t.animation.actual;
            let a = c.lerp(a, s);
            let b = c.lerp(b, s);

            Some((a, b, c, t.color))
        })
    }

    pub fn remove_vertex(&mut self, id: usize) {
        self.vertices.remove_entry(&id);
        self.edges.retain(|_, e| !e.has_vertex(id));
        // TODO faces
    }

    pub fn on_right_click_down(&mut self, pos: &mut TakeOnce<Vec2>) {
        let pos = match pos.take() {
            Some(v) => v,
            _ => return,
        };
        if let Some(id) = self.vertex_at(pos, CLICK_TARGET_SIZE_PIXELS) {
            let n_active_edges = self
                .edges
                .iter()
                .filter(|(_, e)| e.has_vertex(id) && e.is_visible)
                .count();
            let n_edges = self.edges.iter().filter(|(_, e)| e.has_vertex(id)).count();
            if n_active_edges > 0 {
                self.edges
                    .iter_mut()
                    .filter(|(_, e)| e.has_vertex(id))
                    .for_each(|(_, e)| e.is_visible = false);
            } else if n_edges > 0 {
                self.edges.retain(|_, e| !e.has_vertex(id));
            } else {
                self.remove_vertex(id);
            }
        }
    }

    pub fn on_left_click_down(&mut self, pos: Vec2) {
        if let Some(id) = self.vertex_at(pos, CLICK_TARGET_SIZE_PIXELS) {
            if let Some(x) = self.vertices.get_mut(&id) {
                x.is_clicked = true;
            }
        }
    }

    fn get_hovered_vertex(&mut self) -> Option<usize> {
        self.vertices
            .iter_mut()
            .find_map(|(id, v)| v.is_hovered.then(|| *id))
    }

    pub fn get_clicked_vertex(&mut self) -> Option<usize> {
        self.vertices
            .iter_mut()
            .find_map(|(id, v)| v.is_clicked.then(|| *id))
    }

    fn get_edge_mut(&mut self, a: usize, b: usize) -> Option<&mut Edge> {
        let emin = a.min(b);
        let emax = a.max(b);
        self.edges.get_mut(&(emin, emax))
    }

    fn is_edge(&self, a: usize, b: usize) -> bool {
        let (a, b) = (a.min(b), a.max(b));
        self.edges.contains_key(&(a, b))
    }

    fn add_edge(&mut self, a: usize, b: usize, state: bool) {
        if a == b {
            return;
        }

        let min = a.min(b);
        let max = a.max(b);
        let edge = Edge::new(min, max, state);
        self.edges.insert((min, max), edge);
        self.update_triangles();
    }

    pub fn on_left_click_up(&mut self) {
        let clicked = self.get_clicked_vertex();
        let hovered = self.get_hovered_vertex();

        if let Some((c, h)) = clicked.zip(hovered) {
            if let Some(e) = self.get_edge_mut(c, h) {
                e.is_visible = !e.is_visible;
            } else {
                self.add_edge(c, h, false)
            }
        }

        for (_, v) in &mut self.vertices {
            v.is_clicked = false;
        }
    }

    pub fn set_cursor_position(&mut self, p: &mut TakeOnce<Vec2>) {
        let pos = p.take();
        for (_, v) in &mut self.vertices {
            if !v.is_follow() {
                v.is_hovered = false;
            }
            let base_radius = 8.0;
            let extra = if let Some(pos) = pos {
                let d = pos.distance(v.pos);
                7.0 * (1.0 - d / 200.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            v.marker_radius.target = base_radius + extra;
            if v.is_follow() {
                if let Some(p) = pos {
                    v.pos += (p - v.pos) * 0.25;
                }
            }
        }

        if let Some(pos) = pos {
            if let Some(id) = self.vertex_at(pos, CLICK_TARGET_SIZE_PIXELS) {
                if let Some(v) = self.vertices.get_mut(&id) {
                    v.is_hovered = true;
                }
            }
        }

        self.active_line = || -> Option<(usize, Vec2)> {
            let pos = pos?;
            let idx = self.get_clicked_vertex()?;
            Some((idx, pos))
        }();
    }

    pub fn active_line(&self) -> Option<(usize, Vec2)> {
        self.active_line
    }

    fn get_triangle_at(&mut self, p: Vec2) -> Option<&mut Triangle> {
        self.triangles.iter_mut().find_map(|(_, t)| {
            let a = self.vertices.get(&t.a)?.pos;
            let b = self.vertices.get(&t.b)?.pos;
            let c = self.vertices.get(&t.c)?.pos;
            point_in_triangle(p, a, b, c).then(|| t)
        })
    }

    pub fn set_color(&mut self, p: Vec2, color: Srgba) {
        if let Some(t) = self.get_triangle_at(p) {
            t.color = color;
        }
    }

    pub fn step(&mut self) {
        let is_complete = self.is_complete();

        for (id, v) in &mut self.vertices {
            v.visible_count = self
                .edges
                .iter()
                .filter(|(_, e)| (e.a == *id || e.b == *id) && e.is_visible)
                .count();

            v.invisible_count = self
                .edges
                .iter()
                .filter(|(_, e)| (e.a == *id || e.b == *id) && !e.is_visible)
                .count();

            if v.is_clicked && v.is_hovered {
                v.marker_radius.target = 25.0;
            } else if is_complete && v.visible_count > 0 {
                v.marker_radius.target = 0.0;
            } else if v.invisible_count == 0 && v.visible_count > 0 {
                v.marker_radius.target = 3.0;
            }

            v.marker_radius.step();

            v.follow_count = if v.is_clicked && v.is_hovered {
                (v.follow_count + 1).min(MAX_FOLLOW_COUNT)
            } else {
                0
            };
        }

        for (_, e) in &mut self.edges {
            e.length_animation.target = e.is_visible as u8 as f32;
            e.length_animation.step();
            e.thickness_animation.target = if is_complete {
                0.0
            } else if e.is_visible {
                3.0
            } else {
                1.0
            };
            e.thickness_animation.step();
        }

        for (_, t) in &mut self.triangles {
            t.is_visible = is_edge_visible(&self.edges, t.a, t.b)
                && is_edge_visible(&self.edges, t.a, t.c)
                && is_edge_visible(&self.edges, t.b, t.c);

            t.animation.target = t.is_visible as u8 as f32;
            t.animation.step();
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.triangles.is_empty()
            && self.triangles.iter().all(|(_, t)| t.is_visible)
            && self.vertices.iter().all(|(_, v)| v.visible_count > 1)
    }
}

fn is_edge_visible(edges: &HashMap<(usize, usize), Edge>, a: usize, b: usize) -> bool {
    if let Some(e) = edges.get(&(a.min(b), a.max(b))) {
        e.is_visible
    } else {
        false
    }
}

#[derive(Deserialize, Serialize, Default)]
struct PuzzleRepr {
    vertices: HashMap<usize, Vec2>,
    edges: Vec<(usize, usize)>,
    triangles: Vec<(usize, usize, usize, Srgba)>,
}

impl From<PuzzleRepr> for Puzzle {
    fn from(value: PuzzleRepr) -> Self {
        let mut puzzle = Puzzle::empty();
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
            puzzle.add_edge(a, b, rand() < 0.3);
        }

        for (a, b, c, color) in value.triangles {
            if let Some(t) = puzzle.triangles.get_mut(&(a, b, c)) {
                t.color = color;
            }
        }

        puzzle.next_vertex_id = max_id + 1;

        puzzle
    }
}

impl From<&Puzzle> for PuzzleRepr {
    fn from(value: &Puzzle) -> Self {
        let mut repr = PuzzleRepr::default();
        for (id, p) in &value.vertices {
            repr.vertices.insert(*id, p.pos);
        }
        for (_, e) in &value.edges {
            repr.edges.push((e.a, e.b));
        }
        for (_, t) in &value.triangles {
            repr.triangles.push((t.a, t.b, t.c, t.color));
        }
        repr
    }
}

pub fn puzzle_to_file(puzzle: &Puzzle, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repr = PuzzleRepr::from(puzzle);
    let s = serde_yaml::to_string(&repr)?;
    std::fs::write(filepath, s)?;
    Ok(())
}

pub fn puzzle_from_file(
    filepath: impl Into<PathBuf>,
) -> Result<Puzzle, Box<dyn std::error::Error>> {
    let filepath = filepath.into();
    let s = std::fs::read_to_string(filepath)?;
    let repr: PuzzleRepr = serde_yaml::from_str(&s)?;
    Ok(Puzzle::from(repr))
}

pub fn point_in_triangle(test: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let alpha = ((b.y - c.y) * (test.x - c.x) + (c.x - b.x) * (test.y - c.y))
        / ((b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y));
    let beta = ((c.y - a.y) * (test.x - c.x) + (a.x - c.x) * (test.y - c.y))
        / ((b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y));
    let gamma = 1.0 - alpha - beta;
    alpha > 0.0 && beta > 0.0 && gamma > 0.0
}
