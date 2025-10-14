use crate::math::*;
pub use crate::puzzle::*;
pub use bevy::color::palettes::css::*;
pub use bevy::color::Srgba;
pub use bevy_vector_shapes::prelude::*;

pub fn draw_rect(painter: &mut ShapePainter, origin: Vec2, dims: Vec2, z: f32, color: Srgba) {
    painter.reset();
    painter.set_translation((origin + dims / 2.0).extend(z));
    painter.set_color(color);
    painter.rect(dims);
}

pub fn draw_circle(painter: &mut ShapePainter, p: Vec2, z: f32, r: f32, color: Srgba) {
    painter.thickness = 3.0;
    painter.hollow = false;
    painter.set_translation(p.extend(z));
    painter.set_color(color);
    painter.circle(r);
    painter.set_translation(Vec3::ZERO);
}

pub fn draw_hollow_circle(
    painter: &mut ShapePainter,
    p: Vec2,
    z: f32,
    r: f32,
    t: f32,
    color: Srgba,
) {
    painter.thickness = t;
    painter.hollow = true;
    painter.set_translation(p.extend(z));
    painter.set_color(color);
    painter.circle(r + t / 2.0);
    painter.set_translation(Vec3::ZERO);
}

pub fn fill_ring(painter: &mut ShapePainter, p: Vec2, z: f32, ri: f32, ro: f32, color: Srgba) {
    painter.thickness = ro - ri;
    painter.hollow = true;
    painter.set_translation(p.extend(z));
    painter.set_color(color);
    painter.circle(ro);
    painter.set_translation(Vec3::ZERO);
}

pub fn draw_triangle(painter: &mut ShapePainter, a: Vec2, b: Vec2, c: Vec2, z: f32, color: Srgba) {
    painter.set_translation(Vec2::ZERO.extend(z));
    painter.set_color(color);
    painter.triangle(a, b, c);
}

pub fn draw_line(
    painter: &mut ShapePainter,
    a: Vec2,
    b: Vec2,
    z: f32,
    thickness: f32,
    color: Srgba,
) {
    painter.thickness = thickness;
    painter.set_color(color);
    painter.set_translation(Vec2::ZERO.extend(z));
    painter.line(a.extend(0.0), b.extend(0.0));
}

const SNAP_GRID_Z: f32 = 0.08;

pub fn draw_snap_grid(painter: &mut ShapePainter, puzzle: &Puzzle, pos: Option<Vec2>) {
    let r = 200.0;
    let color = LIGHT_GRAY;
    let thickness = 1.0;
    for v in puzzle.vertices() {
        let up = v.pos + Vec2::Y * r;
        let down = v.pos - Vec2::Y * r;
        let right = v.pos + Vec2::X * r;
        let left = v.pos - Vec2::X * r;
        draw_line(painter, left, right, SNAP_GRID_Z, thickness, color);
        draw_line(painter, up, down, SNAP_GRID_Z, thickness, color);

        if let Some(p) = pos {
            if v.pos.distance(p) > r {
                continue;
            }
            let u = p.with_x(v.pos.x);
            let v = p.with_y(v.pos.y);
            draw_circle(painter, u, SNAP_GRID_Z, 3.0, RED);
            draw_circle(painter, v, SNAP_GRID_Z, 3.0, RED);
        }
    }
}
