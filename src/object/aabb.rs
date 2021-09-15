use crate::{material::Material, math::{Ray, Vector3}};

use super::{Hit, Intersect, SceneObject};

/// An axis-aligned box, short for axis-aligned bounding box.
#[derive(Debug, Clone)]
pub struct Aabb {
    pub pos: Vector3,
    pub size: Vector3,
    pub material: Material,
}

impl Aabb {
    pub fn new(pos: Vector3, size: Vector3, material: Material) -> Self {
        Self { pos, size, material }
    }
}

impl Intersect for Aabb {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let ro = ray.origin - self.pos;
        let s = Vector3::new(
            if ray.direction.x < 0. { 1. } else { -1. },
            if ray.direction.y < 0. { 1. } else { -1. },
            if ray.direction.z < 0. { 1. } else { -1. },
        );

        let t1 = ray.inverse() * (-ro + (s * self.size));
        let t2 = ray.inverse() * (-ro - (s * self.size));
        let tn = f64::max(f64::max(t1.x, t1.y), t1.z);
        let tf = f64::min(f64::min(t2.x, t2.y), t2.z);

        if tn > tf || tf < 0. {
            return None;
        }

        let normal = if t1.x > t1.y && t1.x > t1.z {
            Vector3::new(s.x, 0., 0.)
        } else if t1.y > t1.z {
            Vector3::new(0., s.y, 0.)
        } else {
            Vector3::new(0., 0., s.z)
        };

        Some(Hit::new(normal, tn, tf))
    }
}

impl SceneObject for Aabb {
    fn material(&self) -> &Material {
        &self.material
    }
}
