#![allow(unused)]

use crate::math::*;
pub use crate::puzzle::*;
pub use bevy::color::palettes::css::*;
pub use bevy::color::Srgba;
pub use bevy_vector_shapes::prelude::*;

pub fn fill_rect(painter: &mut ShapePainter, origin: Vec2, dims: Vec2, color: Srgba) {
    painter.reset();
    painter.set_translation((origin + dims / 2.0).extend(0.28));
    painter.set_color(color);
    painter.rect(dims);
    painter.hollow = true;
    painter.set_color(BLACK);
    painter.thickness = 2.0;
    painter.rect(dims);
}

pub fn draw_rect(painter: &mut ShapePainter, origin: Vec2, dims: Vec2, t: f32, color: Srgba) {
    painter.reset();
    painter.set_translation((origin + dims / 2.0).extend(1000.0));
    painter.set_color(color);
    painter.hollow = true;
    painter.thickness = t;
    painter.rect(dims + Vec2::splat(t));
}

pub fn draw_circle(painter: &mut ShapePainter, p: Vec2, z: f32, r: f32, color: Srgba) {
    if r < 1.0 {
        return;
    }
    painter.reset();
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
    if r < 1.0 {
        return;
    }
    painter.reset();
    painter.thickness = t;
    painter.hollow = true;
    painter.set_translation(p.extend(z));
    painter.set_color(color);
    painter.circle(r + t / 2.0);
    painter.set_translation(Vec3::ZERO);
}

pub fn fill_ring(painter: &mut ShapePainter, p: Vec2, z: f32, ri: f32, ro: f32, color: Srgba) {
    painter.reset();
    painter.thickness = ro - ri;
    painter.hollow = true;
    painter.set_translation(p.extend(z));
    painter.set_color(color);
    painter.circle(ro);
    painter.set_translation(Vec3::ZERO);
}

pub fn draw_triangle(painter: &mut ShapePainter, a: Vec2, b: Vec2, c: Vec2, z: f32, color: Srgba) {
    painter.reset();
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
    painter.reset();
    painter.thickness = thickness;
    painter.set_color(color);
    painter.set_translation(Vec2::ZERO.extend(z));
    painter.line(a.extend(0.0), b.extend(0.0));
}
