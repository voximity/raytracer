use crate::{
    material::Material,
    math::{Ray, Vector3},
    scene::EPSILON,
};

use super::{Hit, Intersect, SceneObject};

/// A plane.
#[derive(Debug, Clone)]
pub struct Plane {
    /// The origin of the plane.
    pub origin: Vector3,

    /// The normal of the plane.
    pub normal: Vector3,

    /// The plane's material.
    pub material: Material,

    /// The unit by which UVs will be wrapped. For example,
    /// when this value is 2, UVs will wrap every 2 units
    /// in both axes.
    pub uv_wrap: f32,
}

impl Plane {
    pub fn new(origin: Vector3, normal: Vector3, material: Material) -> Self {
        Self {
            origin,
            normal,
            material,
            ..Default::default()
        }
    }
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            origin: Vector3::default(),
            normal: Vector3::new(0., 1., 0.),
            material: Material::default(),
            uv_wrap: 1.,
        }
    }
}

impl Intersect for Plane {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let denom = self.normal.dot(ray.direction);
        if denom.abs() > EPSILON {
            let t = (self.origin - ray.origin).dot(self.normal) / denom;
            if t > 0. {
                let p = ray.along(t);
                // TEMPORARY: use x and z coords to determine uvs
                // in the future this should take into account the
                // plane's normal
                Some(Hit::new(
                    self.normal * -denom.signum(),
                    (t, p),
                    (t, p),
                    if self.normal.z != 0. {
                        (
                            (p.x as f32 / self.uv_wrap).rem_euclid(1.),
                            (p.z as f32 / self.uv_wrap).rem_euclid(1.),
                        )
                    } else {
                        (0., 0.)
                    },
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl SceneObject for Plane {
    fn material(&self) -> &Material {
        &self.material
    }
}
