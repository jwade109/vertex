use crate::math::*;
use bevy::color::palettes::basic::*;
use bevy::prelude::*;
use delaunator::{triangulate, Point};
use rand::seq::IndexedRandom;

pub struct Puzzle {
    vertices: Vec<Vec2>,
    edges: Vec<usize>,
    colors: Vec<Srgba>,
}

fn random_color() -> Srgba {
    let colors = vec![
        AQUA, BLACK, BLUE, FUCHSIA, GRAY, GREEN, LIME, MAROON, NAVY, OLIVE, PURPLE, RED, SILVER,
        TEAL, WHITE, YELLOW,
    ];

    *colors.choose(&mut rand::rng()).unwrap()
}

impl Puzzle {
    pub fn empty() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
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
                x: v.x as f64,
                y: v.y as f64,
            })
            .collect();

        let result = triangulate(&dpoints);
        self.edges = result.triangles;

        let n_triangles = self.edges.len() / 3;

        while self.colors.len() < n_triangles {
            let color = random_color();
            self.colors.push(color);
        }

        while self.colors.len() > n_triangles {
            self.colors.pop();
        }
    }

    pub fn add_point(&mut self, p: Vec2) {
        self.vertices.push(p);
        self.update();
    }

    pub fn randomize(&mut self) {
        self.vertices.clear();
        for _ in 0..5 {
            let v = Vec2::new(rand() * 600.0 + 100.0, rand() * 800.0 - 400.0);
            let q = v.with_x(-v.x);
            self.vertices.push(v);
            self.vertices.push(q);
        }
        self.update();
    }

    pub fn vertices(&self) -> impl Iterator<Item = Vec2> + use<'_> {
        self.vertices.iter().map(|v| *v)
    }

    pub fn vertex_n(&self, n: usize) -> Option<Vec2> {
        self.vertices.get(n).cloned()
    }

    pub fn triangles(&self) -> impl Iterator<Item = (Vec2, Vec2, Vec2, Srgba)> + use<'_> {
        self.edges.windows(3).enumerate().filter_map(|(i, e)| {
            if i % 3 > 0 {
                return None;
            }
            let u1 = *e.get(0)?;
            let u2 = *e.get(1)?;
            let u3 = *e.get(2)?;
            let v1 = self.vertex_n(u1)?;
            let v2 = self.vertex_n(u2)?;
            let v3 = self.vertex_n(u3)?;
            let c = self.colors.get(i / 3)?;
            Some((v1, v2, v3, *c))
        })
    }
}
