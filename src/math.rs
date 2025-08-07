pub use bevy::math::Vec2;

use rand::Rng;

pub fn rand() -> f32 {
    rand::rng().random_range(0.0..=1.0)
}
