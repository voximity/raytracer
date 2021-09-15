use crate::math::{Lerp, Vector3, lerp};

/// A 24-bit color, RGB.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Instantiate a new Color.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Instantiate a new Color from 3 f64s, expected to be in the range 0-1.
    pub fn newf(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: (r.clamp(0., 1.) * 255.0) as u8,
            g: (g.clamp(0., 1.) * 255.0) as u8,
            b: (b.clamp(0., 1.) * 255.0) as u8,
        }
    }

    /// Instantiate a Color from a Vector3. Useful for checking normals.
    pub fn from_normal(n: Vector3) -> Self {
        Self::newf(n.x / 2. + 0.5, n.y / 2. + 0.5, n.z / 2. + 0.5)
    }
}

impl From<Vector3> for Color {
    fn from(v: Vector3) -> Self {
        Self::newf(v.x, v.y, v.z)
    }
}

impl Lerp for Color {
    fn lerp(self, other: Self, t: f64) -> Self {
        Color {
            r: lerp(self.r as f64, other.r as f64, t).clamp(0., 255.) as u8,
            g: lerp(self.g as f64, other.g as f64, t).clamp(0., 255.) as u8,
            b: lerp(self.b as f64, other.b as f64, t).clamp(0., 255.) as u8,
        }
    }
}

/// A material for a scene object. Over time, this struct
/// will be populated with more physical rendering
/// properties.
#[derive(Clone, Debug)]
pub struct Material {
    pub color: Color,
    pub reflectiveness: f64,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: Color::new(255, 255, 255),
            reflectiveness: 0.,
        }
    }
}
