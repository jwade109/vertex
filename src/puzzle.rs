use crate::edge::*;
use crate::math::*;
use crate::triangle::*;
use crate::vertex::*;
use bevy::prelude::*;
use delaunator::{triangulate, Point};
use rand::seq::IndexedRandom;
use std::collections::HashMap;

const CLICK_TARGET_SIZE_PIXELS: f32 = 50.0;

pub struct Puzzle {
    vertices: Vec<Vertex>,
    edges: HashMap<(usize, usize), Edge>,
    triangles: Vec<Triangle>,
    active_line: Option<(usize, Vec2)>,
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
            edges: HashMap::new(),
            triangles: Vec::new(),
            active_line: None,
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

        self.triangles = result
            .triangles
            .windows(3)
            .enumerate()
            .filter_map(|(i, w)| {
                if i % 3 > 0 {
                    return None;
                }

                Some(Triangle::new(w[0], w[1], w[2], random_color()))
            })
            .collect();

        let edges: HashMap<_, _> = (0..result.triangles.len())
            .filter_map(|i| {
                let (i, j) = match i % 3 {
                    0 | 1 => (i, i + 1),
                    _ => (i, i - 2),
                };

                let e1 = *result.triangles.get(i)?;
                let e2 = *result.triangles.get(j)?;

                let emin = e1.min(e2);
                let emax = e1.max(e2);

                if emin == emax {
                    return None;
                }

                Some(((emin, emax), Edge::new(emin, emax, false)))
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
        for _ in 0..10 {
            let v = Vec2::new(random(-1000.0, 1000.0), random(-600.0, 600.0));
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
        self.edges.iter().filter_map(|(_, e)| {
            let v1 = self.vertex_n(e.a)?;
            let v2 = self.vertex_n(e.b)?;
            Some((v1, v2, e))
        })
    }

    pub fn triangles(&self) -> impl Iterator<Item = (Vec2, Vec2, Vec2, Srgba)> + use<'_> {
        self.triangles.iter().filter_map(|t| {
            if t.animation.actual < 0.01 {
                return None;
            }

            let a = self.vertex_n(t.a)?.pos;
            let b = self.vertex_n(t.b)?.pos;
            let c = self.vertex_n(t.c)?.pos;

            let center = (a + b + c) / 3.0;
            let s = t.animation.actual;
            let a = center.lerp(a, s);
            let b = center.lerp(b, s);
            let c = center.lerp(c, s);

            Some((a, b, c, t.color))
        })
    }

    pub fn on_right_click_down(&mut self, pos: Vec2) {
        if let Some(idx) = self.vertex_at(pos, CLICK_TARGET_SIZE_PIXELS) {
            for (_, e) in &mut self.edges {
                if e.a == idx || e.b == idx {
                    e.is_visible = false;
                }
            }
        }
    }

    pub fn on_left_click_down(&mut self, pos: Vec2) {
        if let Some(idx) = self.vertex_at(pos, CLICK_TARGET_SIZE_PIXELS) {
            if let Some(x) = self.vertices.get_mut(idx) {
                x.is_clicked = true;
            }
        }
    }

    fn get_hovered_vertex(&mut self) -> Option<usize> {
        self.vertices
            .iter_mut()
            .enumerate()
            .find_map(|(idx, v)| v.is_hovered.then(|| idx))
    }

    pub fn get_clicked_vertex(&mut self) -> Option<usize> {
        self.vertices
            .iter_mut()
            .enumerate()
            .find_map(|(idx, v)| v.is_clicked.then(|| idx))
    }

    fn get_edge_mut(&mut self, a: usize, b: usize) -> Option<&mut Edge> {
        let emin = a.min(b);
        let emax = a.max(b);
        self.edges.get_mut(&(emin, emax))
    }

    pub fn on_left_click_up(&mut self) {
        let clicked = self.get_clicked_vertex();
        let hovered = self.get_hovered_vertex();

        if let Some((c, h)) = clicked.zip(hovered) {
            if let Some(e) = self.get_edge_mut(c, h) {
                e.is_visible = !e.is_visible;
            }
        }

        for v in &mut self.vertices {
            v.is_clicked = false;
        }
    }

    pub fn set_cursor_position(&mut self, pos: Option<Vec2>) {
        for v in &mut self.vertices {
            v.is_hovered = false;
            let base_radius = 8.0;
            let extra = if let Some(pos) = pos {
                let d = pos.distance(v.pos);
                7.0 * (1.0 - d / 200.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            v.marker_radius.target = base_radius + extra;
        }

        if let Some(pos) = pos {
            if let Some(idx) = self.vertex_at(pos, CLICK_TARGET_SIZE_PIXELS) {
                if let Some(v) = self.vertices.get_mut(idx) {
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

    pub fn step(&mut self) {
        let is_complete = self.is_complete();

        for (idx, v) in self.vertices.iter_mut().enumerate() {
            v.visible_count = self
                .edges
                .iter()
                .filter(|(_, e)| (e.a == idx || e.b == idx) && e.is_visible)
                .count();

            v.invisible_count = self
                .edges
                .iter()
                .filter(|(_, e)| (e.a == idx || e.b == idx) && !e.is_visible)
                .count();

            if v.is_clicked && v.is_hovered {
                v.marker_radius.target = 25.0;
            } else if is_complete {
                v.marker_radius.target = 0.0;
            } else if v.invisible_count == 0 {
                v.marker_radius.target = 3.0;
            }

            v.marker_radius.step();
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

        for t in &mut self.triangles {
            t.is_visible = is_edge_visible(&self.edges, t.a, t.b)
                && is_edge_visible(&self.edges, t.a, t.c)
                && is_edge_visible(&self.edges, t.b, t.c);

            t.animation.target = t.is_visible as u8 as f32;
            t.animation.step();
        }
    }

    pub fn is_complete(&self) -> bool {
        self.triangles.iter().all(|t| t.is_visible)
    }
}

fn is_edge_visible(edges: &HashMap<(usize, usize), Edge>, a: usize, b: usize) -> bool {
    if let Some(e) = edges.get(&(a.min(b), a.max(b))) {
        e.is_visible
    } else {
        false
    }
}
