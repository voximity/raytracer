mod matrix;
mod ray;
mod vector;

pub use matrix::*;
pub use ray::*;
pub use vector::*;

/// Linearly interpolate between two values.
pub fn lerp(a: f64, b: f64, c: f64) -> f64 {
    a + (b - a) * c
}

/// A type that can be linearly interpolated between two values of itself.
pub trait Lerp {
    fn lerp(self, other: Self, t: f64) -> Self;
}
