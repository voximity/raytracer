use std::f32::consts::PI;

use crate::{
    material::Material,
    math::{Ray, Vector3},
};

use super::{Hit, Intersect, SceneObject};

/// A sphere.
#[derive(Debug, Clone)]
pub struct Sphere {
    /// The origin of the sphere.
    pub origin: Vector3,

    /// The radius of the sphere.
    pub radius: f64,

    /// The material of the sphere.
    pub material: Material,
}

impl Sphere {
    pub fn new(origin: Vector3, radius: f64, material: Material) -> Self {
        Self {
            origin,
            radius,
            material,
        }
    }
}

impl Intersect for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let rad2 = self.radius.powi(2);
        let l = self.origin - ray.origin;
        let t2 = l.dot(ray.direction);
        if t2 < 0.0 {
            return None;
        }

        let d2 = l.dot(l) - t2 * t2;
        if d2 > rad2 {
            return None;
        }

        let t3 = (rad2 - d2).sqrt();
        let t0 = t2 - t3;
        let t1 = t2 + t3;

        let vtn = ray.along(t0);
        let vtf = ray.along(t1);

        // TEMPORARY: i really don't feel like doing uv calculation for spheres right now
        let uv = (
            (1. - (vtn.z - self.origin.z).atan2(vtn.x - self.origin.x) as f32 + PI) / (PI * 2.),
            0.5 * (PI / 4. * (self.origin.y - vtn.y) as f32 / self.radius as f32).tan() + 0.5,
        );

        Some(Hit::new(
            (vtn - self.origin).normalize(),
            (t0, vtn),
            (t1, vtf),
            uv,
        ))
    }
}

impl SceneObject for Sphere {
    fn material(&self) -> &Material {
        &self.material
    }
}
