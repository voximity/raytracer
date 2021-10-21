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

impl Lerp for f64 {
    fn lerp(self, other: Self, t: f64) -> Self {
        lerp(self, other, t)
    }
}

/// An axis.
#[derive(Debug, Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// Calculate the refraction vectors based on a ray, a normal, and the two IORs.
pub fn refraction_vec(
    in_ray: &Ray,
    normal: Vector3,
    from_ior: f64,
    to_ior: f64,
) -> Option<Vector3> {
    let n = from_ior / to_ior;
    let cos_i = -normal.dot(in_ray.direction);
    let sin_t2 = n * n * (1. - cos_i * cos_i);
    if sin_t2 > 1. {
        return None;
    }

    let cos_t = (1. - sin_t2).sqrt();
    Some(in_ray.direction * n + normal * (n * cos_i - cos_t))
}
