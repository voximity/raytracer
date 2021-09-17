mod aabb;
mod mesh;
mod plane;
mod sphere;

use crate::{
    material::Material,
    math::{Ray, Vector3},
};

pub use aabb::*;
pub use mesh::*;
pub use plane::*;
pub use sphere::*;

/// The result of a ray intersection, including hit location data and UV data.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    /// The normal (as perceived) of the hit.
    pub normal: Vector3,

    /// The near t of the hit.
    pub near: f64,

    /// The near point of the hit.
    pub vnear: Vector3,

    /// The far t of the hit.
    pub far: f64,

    /// The far point of the hit.
    pub vfar: Vector3,

    /// The UV coordinates of the hit, for texture polling.
    pub uv: (f32, f32),
}

impl Hit {
    pub fn new(
        normal: Vector3,
        (near, vnear): (f64, Vector3),
        (far, vfar): (f64, Vector3),
        uv: (f32, f32),
    ) -> Self {
        Self {
            normal,
            near,
            vnear,
            far,
            vfar,
            uv,
        }
    }

    pub fn pos(&self, ray: &Ray) -> Vector3 {
        ray.along(self.near)
    }
}

/// A trait that represents any type that can be intersected by a Ray.
pub trait Intersect {
    /// Find the intersection, if any, between the ray provided and this shape.
    fn intersect(&self, ray: &Ray) -> Option<Hit>;
}

/// A trait that represents any type that is a scene object, and can thus be viewed in the final render.
pub trait SceneObject: Intersect + Send + Sync {
    /// Grab this scene object's material.
    fn material(&self) -> &Material;
}
