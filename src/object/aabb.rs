use crate::{
    material::Material,
    math::{Ray, Vector3},
};

use super::{Hit, Intersect, SceneObject};

/// A type that is solely used for intersection with rays.
/// It is used so that there is less memory overhead than
/// a typical `Aabb`, which also must return material data.
#[derive(Debug, Clone, Default)]
pub struct AabbIntersector {
    pub pos: Vector3,
    pub size: Vector3,
}

impl Intersect for AabbIntersector {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let ro = ray.origin - self.pos;
        let s = Vector3::new(
            -ray.direction.x.signum(),
            -ray.direction.y.signum(),
            -ray.direction.z.signum(),
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

        let pn = ray.along(tn);
        let pf = ray.along(tf);

        let pns = (Vector3::new(1., 1., 1.) - normal.abs()) * (pn - self.pos) / self.size;

        #[rustfmt::skip]
        let uv: (f64, f64) = match normal {
            Vector3 { y, .. } if y == 1. => (pns.x, pns.z),
            Vector3 { y, .. } if y == -1. => (-pns.x, -pns.z),
            Vector3 { x, .. } if x == 1. => (-pns.z, -pns.y),
            Vector3 { x, .. } if x == -1. => (pns.z, -pns.y),
            Vector3 { z, .. } if z == 1. => (pns.x, -pns.y),
            Vector3 { z, .. } if z == -1. => (-pns.x, -pns.y),
            _ => (0., 0.),
        };

        Some(Hit::new(
            normal,
            (tn, pn),
            (tf, pf),
            (uv.0 as f32 * 0.5 + 0.5, uv.1 as f32 * 0.5 + 0.5),
        ))
    }
}

/// An axis-aligned box, short for axis-aligned bounding box.
#[derive(Debug, Clone, Default)]
pub struct Aabb {
    intersector: AabbIntersector,
    pub material: Material,
}

impl Aabb {
    /// Instantiate a new `Aabb` scene object. Internally, this also instantiates
    /// an `AabbIntersector`, to which this type wraps around.
    pub fn new(pos: Vector3, size: Vector3, material: Material) -> Self {
        Self {
            intersector: AabbIntersector { pos, size },
            material,
        }
    }

    /// Gets the `pos` of the inner `AabbIntersector`.
    pub fn pos(&self) -> Vector3 {
        self.intersector.pos
    }

    /// Gets the `size` of the inner `AabbIntersector`.
    pub fn size(&self) -> Vector3 {
        self.intersector.size
    }

    /// Consumes this `Aabb`, returning the inner `AabbIntersector`.
    pub fn into_inner(self) -> AabbIntersector {
        self.intersector
    }
}

impl Intersect for Aabb {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        self.intersector.intersect(ray)
    }
}

impl SceneObject for Aabb {
    fn material(&self) -> &Material {
        &self.material
    }
}
