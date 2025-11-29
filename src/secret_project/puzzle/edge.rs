#![allow(unused)]

use crate::*;

#[derive(Default)]
pub struct Edges(pub HashSet<(usize, usize)>);

pub fn normalize_edge(a: usize, b: usize) -> (usize, usize) {
    let min = a.min(b);
    let max = a.max(b);
    (min, max)
}

impl Edges {
    pub fn add_edge(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        let key = normalize_edge(a, b);
        self.0.insert(key);
    }

    pub fn remove_edge(&mut self, a: usize, b: usize) {
        let key = normalize_edge(a, b);
        self.0.remove(&key);
    }

    pub fn toggle(&mut self, a: usize, b: usize) {
        if self.is_edge(a, b) {
            self.remove_edge(a, b);
        } else {
            self.add_edge(a, b);
        }
    }

    pub fn is_edge(&self, a: usize, b: usize) -> bool {
        let key = normalize_edge(a, b);
        self.0.contains(&key)
    }

    pub fn remove_vertex(&mut self, id: usize) {
        self.0.retain(|(a, b)| *a != id && *b != id);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}
