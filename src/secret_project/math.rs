pub use bevy::math::Vec2;
pub use bevy::math::Vec3;

use rand::Rng;

pub fn rand() -> f32 {
    rand::rng().random_range(0.0..=1.0)
}

pub fn randint(a: i32, b: i32) -> i32 {
    rand::rng().random_range(a..b)
}

pub fn random(a: f32, b: f32) -> f32 {
    rand::rng().random_range(a..=b)
}

#[derive(Debug, Clone, Copy)]
pub struct Lpf {
    pub target: f32,
    pub actual: f32,
    pub alpha: f32,
}

impl Lpf {
    pub fn new(target: f32, actual: f32, alpha: f32) -> Self {
        Self {
            target,
            actual,
            alpha,
        }
    }

    pub fn step(&mut self) {
        self.actual += (self.target - self.actual) * self.alpha;
    }
}
