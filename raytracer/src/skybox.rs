use crate::{
    material::Color,
    math::{blerp, Ray, Vector3},
    object::{AabbIntersector, Hit, Intersect},
};

/// Any type that can be used as a skybox for a scene.
pub trait Skybox: Send + Sync {
    /// The color a ray should produce for the skybox.
    fn ray_color(&self, ray: &Ray) -> Color;
}

/// A solid color skybox.
#[derive(Debug, Clone)]
pub struct Solid(pub Color);

impl Skybox for Solid {
    fn ray_color(&self, _: &Ray) -> Color {
        self.0
    }
}

/// A skybox that creates a color from the ray's direction as if it were a normal.
#[derive(Debug, Clone)]
pub struct Normal;

impl Skybox for Normal {
    fn ray_color(&self, ray: &Ray) -> Color {
        Color::from_normal(ray.direction)
    }
}

/// A skybox derived from a cubemap image, shaped as a cross angled 90 degrees CCW.
#[derive(Debug, Clone)]
pub struct Cubemap {
    /// The AABB intersector used to find the UV and normal of a ray striking the cubemap.
    aabb: AabbIntersector,

    /// The texture to poll colors from.
    tex: image::RgbImage,

    /// The sidelength of one cubemap side.
    cell_size: u32,
}

impl Cubemap {
    /// Create a new cubemap from a texture.
    pub fn new(tex: image::RgbImage) -> Self {
        let csw = tex.width() / 4;
        let csh = tex.height() / 3;
        assert!(csw == csh);

        Cubemap {
            aabb: AabbIntersector {
                pos: Vector3::default(),
                size: Vector3::new(0.5, 0.5, 0.5),
            },
            tex,
            cell_size: csw,
        }
    }

    /// Grab a pixel from the cubemap at a certain cell.
    pub fn poll_tex(&self, cx: u32, cy: u32, x: f32, y: f32) -> Color {
        let x = (cx * self.cell_size) as f64 + x as f64 * (self.cell_size - 1) as f64;
        let y = (cy * self.cell_size) as f64 + y as f64 * (self.cell_size - 1) as f64;
        if x % 1. == 0. && y % 1. == 0. {
            self.tex.get_pixel(x as u32, y as u32).0.into()
        } else {
            let (xmin, xmax) = (x.floor(), x.ceil());
            let (ymin, ymax) = (y.floor(), y.ceil());
            let c00: Color = self.tex.get_pixel(xmin as u32, ymin as u32).0.into();
            let c10: Color = self.tex.get_pixel(xmax as u32, ymin as u32).0.into();
            let c01: Color = self.tex.get_pixel(xmin as u32, ymax as u32).0.into();
            let c11: Color = self.tex.get_pixel(xmax as u32, ymax as u32).0.into();
            blerp(
                (x - xmin) / (xmax - xmin),
                (y - ymin) / (ymax - ymin),
                c00.into(),
                c10.into(),
                c01.into(),
                c11.into(),
            )
            .into()
        }
    }
}

impl Skybox for Cubemap {
    fn ray_color(&self, ray: &Ray) -> Color {
        let ray = Ray::new(ray.direction * 2., -ray.direction);
        let Hit { normal, uv, .. } = self.aabb.intersect(&ray).unwrap();

        let (cx, cy) = if normal.z == -1. {
            (1, 1)
        } else if normal.x == -1. {
            (2, 1)
        } else if normal.x == 1. {
            (0, 1)
        } else if normal.z == 1. {
            (3, 1)
        } else if normal.y == 1. {
            (1, 0)
        } else if normal.y == -1. {
            (1, 2)
        } else {
            unreachable!();
        };

        let uv = if normal.y == 1. {
            (1. - uv.0, 1. - uv.1)
        } else if normal.y == -1. {
            (uv.0, 1. - uv.1)
        } else {
            uv
        };

        self.poll_tex(cx, cy, uv.0, uv.1)
    }
}
