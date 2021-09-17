mod matrix;
mod ray;
mod vector;

use std::ops::Range;

pub use matrix::*;
pub use ray::*;
pub use vector::*;

/// Remap a number from one range to another.
pub fn remap(t: f64, a: Range<f64>, b: Range<f64>) -> f64 {
    (t - a.start) * ((b.end - b.start) / (a.end - a.start)) + b.start
}

/// Linearly interpolate between two values.
pub fn lerp(a: f64, b: f64, c: f64) -> f64 {
    a + (b - a) * c
}

/// A type that can be linearly interpolated between two values of itself.
pub trait Lerp {
    fn lerp(self, other: Self, t: f64) -> Self;
}
