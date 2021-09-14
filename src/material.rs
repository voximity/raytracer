use crate::math::Vector3;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
        assert!((0.0..=1.0).contains(&r));
        assert!((0.0..=1.0).contains(&g));
        assert!((0.0..=1.0).contains(&b));

        Self {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        }
    }

    /// Instantiate a Color from a Vector3. Useful for checking normals.
    pub fn from_normal(n: Vector3) -> Self {
        Self::newf(n.x / 2. + 0.5, n.y / 2. + 0.5, n.z / 2. + 0.5)
    }
}

#[derive(Clone, Debug)]
pub struct Material {
    pub color: Color,
}
