use crate::puzzle::Puzzle;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub const GRID_SIZE: f32 = 200.0;

pub fn to_grid(p: Vec2) -> IVec2 {
    let p = p / GRID_SIZE;
    p.floor().as_ivec2()
}

pub fn grid_bounds(p: IVec2) -> (Vec2, Vec2) {
    let lower = p.as_vec2() * GRID_SIZE;
    let upper = (p + IVec2::ONE).as_vec2() * GRID_SIZE;
    (lower, upper)
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

#[derive(Resource, Default)]
pub struct SpatialLookup {
    cells: HashMap<IVec2, HashSet<usize>>,
}

impl SpatialLookup {
    fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn occupied(&self) -> impl Iterator<Item = IVec2> + use<'_> {
        self.cells.iter().map(|(g, _)| *g)
    }

    pub fn lup(&self, g: IVec2) -> Option<&HashSet<usize>> {
        self.cells.get(&g)
    }

    fn update(&mut self, id: usize, p: Vec2) {
        let g = to_grid(p);
        if let Some(g) = self.cells.get_mut(&g) {
            g.insert(id);
        } else {
            let set = HashSet::from([id]);
            self.cells.insert(g, set);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_conversions() {
        assert_eq!(to_grid(Vec2::new(40.0, 130.0)), IVec2::ZERO);
        assert_eq!(to_grid(Vec2::new(40.0, 210.0)), IVec2::new(0, 1));
        assert_eq!(to_grid(Vec2::new(340.0, 210.0)), IVec2::new(1, 1));
        assert_eq!(to_grid(Vec2::new(920.0, 500.0)), IVec2::new(4, 2));

        assert_eq!(to_grid(Vec2::new(-40.0, -130.0)), IVec2::new(-1, -1));
        assert_eq!(to_grid(Vec2::new(-290.0, -430.0)), IVec2::new(-2, -3));
    }
}

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpatialLookup::default())
            .add_systems(Update, update_lut);
    }
}

fn update_lut(puzzle: Single<&Puzzle>, mut lut: ResMut<SpatialLookup>) {
    lut.clear();
    for (id, vertex) in puzzle.vertices() {
        lut.update(id, vertex.pos);
    }
}
