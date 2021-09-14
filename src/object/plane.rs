use crate::{material::Material, math::{Ray, Vector3}, scene::EPSILON};

use super::{Hit, Intersect, SceneObject};

#[derive(Debug, Clone)]
pub struct Plane {
    pub origin: Vector3,
    pub normal: Vector3,
    pub material: Material,
}

impl Plane {
    pub fn new(origin: Vector3, normal: Vector3, material: Material) -> Self {
        Self {
            origin,
            normal,
            material,
        }
    }
}

impl Intersect for Plane {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let denom = self.normal.dot(ray.direction);
        if denom.abs() > EPSILON {
            let t = (self.origin - ray.origin).dot(self.normal) / denom;
            if t > 0. {
                Some(Hit::new(self.normal * -denom.signum(), t, t))
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
