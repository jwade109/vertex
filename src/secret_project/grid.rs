#![allow(unused)]

use crate::*;

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub const GRID_SIZE: f32 = 100.0;

pub fn to_grid(p: Vec2) -> IVec2 {
    let p = p / GRID_SIZE;
    p.floor().as_ivec2()
}

pub fn grid_bounds(g: IVec2) -> (Vec2, Vec2) {
    let lower = g.as_vec2() * GRID_SIZE;
    let upper = (g + IVec2::ONE).as_vec2() * GRID_SIZE;
    (lower, upper)
}

pub fn grid_center(g: IVec2) -> Vec2 {
    let (lower, upper) = grid_bounds(g);
    (upper + lower) / 2.0
}

pub fn local_quad(p: Vec2) -> [IVec2; 4] {
    let g = to_grid(p);
    let bounds = grid_bounds(g);
    let l = p - bounds.0;
    let x = l.x > GRID_SIZE / 2.0;
    let y = l.y > GRID_SIZE / 2.0;
    match (x, y) {
        (true, true) => [g, g + IVec2::X, g + IVec2::Y, g + IVec2::ONE],
        (true, false) => [g, g + IVec2::X, g - IVec2::Y, g + IVec2::new(1, -1)],
        (false, true) => [g, g - IVec2::X, g + IVec2::Y, g + IVec2::new(-1, 1)],
        (false, false) => [g, g - IVec2::X, g - IVec2::Y, g - IVec2::ONE],
    }
}

pub fn line_to_grid(p: Vec2, q: Vec2) -> Vec<IVec2> {
    todo!()
}

pub fn grids_in_radius(p: Vec2, r: f32) -> Vec<IVec2> {
    let center = to_grid(p);
    let offset = (r / GRID_SIZE).ceil() as i32;

    let mut ret = Vec::new();
    for xoff in -offset..=offset {
        for yoff in -offset..=offset {
            let g = center + IVec2::new(xoff, yoff);
            let gc = grid_center(g);
            if gc.distance(p) > (GRID_SIZE / 2.0f32.sqrt()) + r {
                continue;
            }
            ret.push(g);
        }
    }

    ret
}

#[derive(Resource, Default)]
pub struct SpatialLookup {
    vertex_cells: HashMap<IVec2, HashSet<usize>>,
    edge_cells: HashMap<IVec2, HashSet<(usize, usize)>>,
}

fn update_hashset<T: std::cmp::Eq + std::hash::Hash>(
    map: &mut HashMap<IVec2, HashSet<T>>,
    key: IVec2,
    val: T,
) {
    if let Some(g) = map.get_mut(&key) {
        g.insert(val);
    } else {
        let set = HashSet::from([val]);
        map.insert(key, set);
    }
}

impl SpatialLookup {
    fn clear(&mut self) {
        self.vertex_cells.clear();
        self.edge_cells.clear();
    }

    pub fn occupied_vertex(&self) -> impl Iterator<Item = IVec2> + use<'_> {
        self.vertex_cells.iter().map(|(g, _)| *g)
    }

    pub fn occupied_edge(&self) -> impl Iterator<Item = IVec2> + use<'_> {
        self.edge_cells.iter().map(|(g, _)| *g)
    }

    pub fn lup_vertex(&self, g: IVec2) -> Option<&HashSet<usize>> {
        self.vertex_cells.get(&g)
    }

    pub fn lup_edge(&self, g: IVec2) -> Option<&HashSet<(usize, usize)>> {
        self.edge_cells.get(&g)
    }

    fn update_vertex(&mut self, id: usize, p: Vec2) {
        let g = to_grid(p);
        update_hashset(&mut self.vertex_cells, g, id);
    }

    fn update_edge(&mut self, u: (usize, Vec2), v: (usize, Vec2)) {
        let center = (u.1 + v.1) / 2.0;
        let g = to_grid(center);
        update_hashset(&mut self.edge_cells, g, (u.0, v.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_conversions() {
        assert_eq!(to_grid(Vec2::new(40.0, 30.0)), IVec2::ZERO);
        assert_eq!(to_grid(Vec2::new(40.0, 210.0)), IVec2::new(0, 2));
        assert_eq!(to_grid(Vec2::new(340.0, 210.0)), IVec2::new(3, 2));
        assert_eq!(to_grid(Vec2::new(920.0, 500.0)), IVec2::new(9, 5));

        assert_eq!(to_grid(Vec2::new(-40.0, -130.0)), IVec2::new(-1, -2));
        assert_eq!(to_grid(Vec2::new(-290.0, -430.0)), IVec2::new(-3, -5));
    }
}

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpatialLookup::default()).add_systems(
            Update,
            (
                update_lut,
                draw_occupied_cells.run_if(not(in_state(EditorMode::Play))),
            ),
        );
    }
}

fn update_lut(puzzle: Single<&Puzzle>, mut lut: ResMut<SpatialLookup>) {
    lut.clear();
    for (id, vertex) in puzzle.vertices() {
        lut.update_vertex(id, vertex.pos);
    }
    for (a, u, b, v) in puzzle.solution_edges() {
        lut.update_edge((a, u.pos), (b, v.pos));
    }
}

fn draw_occupied_cells(mut painter: ShapePainter, lut: Res<SpatialLookup>) {
    for g in lut.occupied_vertex() {
        draw_grid(&mut painter, g, 2.0, GRAY.with_alpha(0.2));
    }
}
