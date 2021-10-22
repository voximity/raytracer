use crate::math::{lerp, Lerp, Vector3};

/// A 24-bit color, RGB.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn white() -> Self {
        Self::new(255, 255, 255)
    }

    pub fn black() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn red() -> Self {
        Self::new(255, 0, 0)
    }

    pub fn green() -> Self {
        Self::new(0, 255, 0)
    }

    pub fn blue() -> Self {
        Self::new(0, 0, 255)
    }

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
        rgb.0.into()
    }
}

impl From<[u8; 3]> for Color {
    fn from(slice: [u8; 3]) -> Self {
        Self::new(slice[0], slice[1], slice[2])
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

    /// A texture that is a 2x2 checkerboard of two colors.
    Checkerboard(Color, Color),
}

impl Texture {
    pub fn at(&self, (u, v): (f32, f32)) -> Color {
        match self {
            Self::Solid(color) => *color,
            Self::Image(image) => {
                let (w, h) = (image.width() as f32, image.height() as f32);
                image
                    .get_pixel(
                        (u * w).clamp(0., w - 1.) as u32,
                        (v * h).clamp(0., h - 1.) as u32,
                    )
                    .to_owned()
                    .into()
            }
            Self::Checkerboard(col_a, col_b) => match (u > 0.5, v > 0.5) {
                (false, false) => *col_a,
                (true, false) => *col_b,
                (false, true) => *col_b,
                (true, true) => *col_a,
            },
        }
    }
}

/// A material for a scene object. Over time, this struct
/// will be populated with more physical rendering
/// properties.
#[derive(Debug, Clone)]
pub struct Material {
    /// The texture of this material.
    pub texture: Texture,

    /// The reflectiveness (0 to 1) of this material.
    pub reflectiveness: f64,

    /// The transparency of this object. At N=1, the object is completely transparent. At N=0, the object is completely opaque.
    pub transparency: f64,

    /// The index of refraction of this material. Higher numbers are more affected by refraction.
    /// At IOR=1, light passes through perfectly.
    pub ior: f64,

    /// The emissivity of the material. At 0, it is not emissive at all. At 1, it is not affected by lighting
    /// at all.
    pub emissivity: f64,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            texture: Texture::Solid(Color::new(255, 255, 255)),
            reflectiveness: 0.,
            transparency: 0.,
            ior: 1.3,
            emissivity: 0.,
        }
    }
}
