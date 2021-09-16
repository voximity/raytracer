use crate::math::{lerp, Lerp, Vector3};

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

    /// Instantiate a Color from HSV values.
    ///
    /// H is expected to be 0 <= H <= 360.
    pub fn hsv(h: f32, sat: u8, val: u8) -> Self {
        let s = sat as f32 / 255.;
        let v = val as f32 / 255.;
        let c = s * v;
        let x = c * (1. - ((h / 60.) % 2. - 1.).abs());
        let _m = v - c;
        let (r, g, b) = if h >= 0. && h < 60. {
            (c, x, 0.)
        } else if h >= 60. && h < 120. {
            (x, c, 0.)
        } else if h >= 120. && h < 180. {
            (0., c, x)
        } else if h >= 180. && h < 240. {
            (0., x, c)
        } else if h >= 240. && h < 300. {
            (x, 0., c)
        } else {
            (c, 0., x)
        };

        Self::newf(r as f64, g as f64, b as f64)
    }
}

impl From<Vector3> for Color {
    fn from(v: Vector3) -> Self {
        Self::newf(v.x, v.y, v.z)
    }
}

impl From<image::Rgb<u8>> for Color {
    fn from(rgb: image::Rgb<u8>) -> Self {
        Self::new(rgb.0[0], rgb.0[1], rgb.0[2])
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

/// A texture for a material.
#[derive(Clone, Debug)]
pub enum Texture {
    /// A texture that is just a solid color.
    Solid(Color),

    /// A texture that is an image. UVs will be used to pull the proper pixel.
    Image(image::RgbImage),
}

impl Texture {
    pub fn at(&self, (u, v): (f32, f32)) -> Color {
        match self {
            Self::Solid(color) => *color,
            Self::Image(image) => {
                let (w, h) = (image.width() as f32, image.height() as f32);
                image
                    .get_pixel((u * w).clamp(0., w - 1.) as u32, (v * h).clamp(0., h - 1.) as u32)
                    .to_owned()
                    .into()
            }
        }
    }
}

/// A material for a scene object. Over time, this struct
/// will be populated with more physical rendering
/// properties.
#[derive(Clone, Debug)]
pub struct Material {
    pub texture: Texture,
    pub reflectiveness: f64,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            texture: Texture::Solid(Color::new(255, 255, 255)),
            reflectiveness: 0.,
        }
    }
}
