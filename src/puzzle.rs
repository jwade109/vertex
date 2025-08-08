use crate::edge::*;
use crate::math::*;
use crate::vertex::*;
use bevy::prelude::*;
use delaunator::{triangulate, Point};
use rand::seq::IndexedRandom;

pub struct Puzzle {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    triangles: Vec<usize>,
    colors: Vec<Srgba>,
}

fn random_color() -> Srgba {
    let colors = vec!["3c4a74", "5c8d8d", "6caeb3", "9bc2bb", "a4b2ca"];
    let s = *colors.choose(&mut rand::rng()).unwrap();
    Srgba::hex(s).unwrap()
}

impl Puzzle {
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            triangles: Vec::new(),
            colors: Vec::new(),
        }
    }

    pub fn new() -> Self {
        let mut s = Self::empty();
        s.randomize();
        s
    }

    fn update(&mut self) {
        let dpoints: Vec<_> = self
            .vertices
            .iter()
            .map(|v| Point {
                x: v.pos.x as f64,
                y: v.pos.y as f64,
            })
            .collect();

        let result = triangulate(&dpoints);
        self.triangles = result.triangles;

        let n_triangles = self.triangles.len() / 3;

        while self.colors.len() < n_triangles {
            let color = random_color();
            self.colors.push(color);
        }

        while self.colors.len() > n_triangles {
            self.colors.pop();
        }

        let edges: Vec<_> = (0..self.triangles.len())
            .filter_map(|i| {
                let (i, j) = match i % 3 {
                    0 | 1 => (i, i + 1),
                    _ => (i, i - 2),
                };

                let e1 = *self.triangles.get(i)?;
                let e2 = *self.triangles.get(j)?;
                if e1 >= e2 {
                    return None;
                }

                let v1 = self.vertices.get(e1)?;
                let v2 = self.vertices.get(e2)?;
                let hidden = v1.hidden || v2.hidden;

                Some(Edge::new(e1, e2, !hidden))
            })
            .collect();

        self.edges = edges;
    }

    pub fn add_point(&mut self, p: Vec2) {
        if self.vertices.iter().any(|q| q.pos.distance(p) < 60.0) {
            return;
        }

        self.vertices.push(Vertex::new(p));
        self.update();
    }

    pub fn randomize(&mut self) {
        self.vertices.clear();
        for _ in 0..40 {
            let v = Vec2::new(random(-1000.0, 1000.0), random(-600.0, 600.0));
            let v = if v.length() > 600.0 {
                v.normalize_or_zero() * 600.0 * random(0.95, 1.1)
            } else {
                v
            };
            self.add_point(v);
        }
        self.update();
    }

    pub fn vertices(&self) -> impl Iterator<Item = &Vertex> + use<'_> {
        self.vertices.iter()
    }

    pub fn vertex_n(&self, n: usize) -> Option<&Vertex> {
        self.vertices.get(n)
    }

    fn vertex_n_if_showing(&self, n: usize) -> Option<&Vertex> {
        self.vertices
            .get(n)
            .map(|v| (!v.hidden).then(|| v))
            .flatten()
    }

    pub fn vertex_at(&self, p: Vec2, max_radius: f32) -> Option<usize> {
        let mut res = None;
        for (i, v) in self.vertices.iter().enumerate() {
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

        res.map(|(i, _)| i)
    }

    pub fn edges(&self) -> impl Iterator<Item = (&Vertex, &Vertex, &Edge)> + use<'_> {
        self.edges.iter().filter_map(|e| {
            let v1 = self.vertex_n(e.a)?;
            let v2 = self.vertex_n(e.b)?;
            Some((v1, v2, e))
        })
    }

    pub fn triangles(&self) -> impl Iterator<Item = (Vec2, Vec2, Vec2, Srgba)> + use<'_> {
        self.triangles.windows(3).enumerate().filter_map(|(i, e)| {
            if i % 3 > 0 {
                return None;
            }
            let u1 = *e.get(0)?;
            let u2 = *e.get(1)?;
            let u3 = *e.get(2)?;
            let v1 = self.vertex_n_if_showing(u1)?.pos;
            let v2 = self.vertex_n_if_showing(u2)?.pos;
            let v3 = self.vertex_n_if_showing(u3)?.pos;
            let c = self.colors.get(i / 3)?;
            Some((v1, v2, v3, *c))
        })
    }

    pub fn toggle_vertex(&mut self, idx: usize) {
        if let Some(x) = self.vertices.get_mut(idx) {
            x.hidden = !x.hidden;
            x.marker_radius.actual = 30.0;
        }
    }

    pub fn try_toggle_vertex_at(&mut self, pos: Vec2) {
        if let Some(idx) = self.vertex_at(pos, 50.0) {
            self.toggle_vertex(idx);
        }
    }

    pub fn set_cursor_position(&mut self, pos: Option<Vec2>) {
        for v in &mut self.vertices {
            let base_radius = if v.hidden { 4.0 } else { 9.0 };
            let extra = if let Some(pos) = pos {
                let d = pos.distance(v.pos);
                7.0 * (1.0 - d / 200.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            v.marker_radius.target = base_radius + extra;
        }

        for e in &mut self.edges {
            let (a, b) = match (self.vertices.get(e.a), self.vertices.get(e.b)) {
                (Some(a), Some(b)) => (a, b),
                _ => continue,
            };

            let hidden = a.hidden || b.hidden;
            e.portion.target = if hidden { 0.0 } else { 1.0 };
        }
    }

    pub fn step(&mut self) {
        for v in &mut self.vertices {
            v.marker_radius.step();
        }

        for e in &mut self.edges {
            e.portion.step();
        }
    }
}
