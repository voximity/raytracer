use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};

use crate::material::Color;

use super::{lerp, Lerp};

/// A vector in 3D space.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    /// Instantiate a new Vector3.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Instantiate a new Vector3 pointing up.
    pub fn up() -> Self {
        Self {
            x: 0.,
            y: 1.,
            z: 0.,
        }
    }

    /// Instantiate a new Vector3 pointing forward.
    pub fn forward() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: -1.,
        }
    }

    /// Instantiate a new Vector3 pointing right.
    pub fn right() -> Self {
        Self {
            x: 1.,
            y: 0.,
            z: 0.,
        }
    }

    /// Find the dot product between two Vector3s.
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross two Vector3s.
    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: -self.x * other.z + self.z * other.x,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Find the magnitude of this Vector3.
    pub fn magnitude(self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    /// Normalize this Vector3 by dividing it by its own magnitude.
    pub fn normalize(self) -> Self {
        self / self.magnitude()
    }

    /// Inverse this vector.
    pub fn inverse(self) -> Self {
        Self {
            x: 1. / self.x,
            y: 1. / self.y,
            z: 1. / self.z,
        }
    }

    /// Get the absolute value of this vector.
    pub fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul for Vector3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<f64> for Vector3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl MulAssign<f64> for Vector3 {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div for Vector3 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl Div<f64> for Vector3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Neg for Vector3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl From<Color> for Vector3 {
    fn from(color: Color) -> Self {
        Self {
            x: color.r as f64 / 255.,
            y: color.g as f64 / 255.,
            z: color.b as f64 / 255.,
        }
    }
}

impl Lerp for Vector3 {
    fn lerp(self, other: Self, t: f64) -> Self {
        Self {
            x: lerp(self.x, other.x, t),
            y: lerp(self.y, other.y, t),
            z: lerp(self.z, other.z, t),
        }
    }
}
