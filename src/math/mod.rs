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

/// Bilinearly interpolate between four vectors.
pub fn blerp(tx: f64, ty: f64, c00: Vector3, c10: Vector3, c01: Vector3, c11: Vector3) -> Vector3 {
    let a = c00 * (1. - tx) + c10 * tx;
    let b = c01 * (1. - tx) + c11 * tx;
    a * (1. - ty) + b * ty
}

/// A type that can be linearly interpolated between two values of itself.
pub trait Lerp {
    fn lerp(self, other: Self, t: f64) -> Self;
}
