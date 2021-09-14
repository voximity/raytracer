use crate::math::{Ray, Vector3};

/// The result of a ray intersection.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub normal: Vector3,
    pub near: f64,
    pub far: f64,
}

impl Hit {
    pub fn new(normal: Vector3, near: f64, far: f64) -> Self {
        Self { normal, near, far }
    }

    pub fn pos(&self, ray: &Ray) -> Vector3 {
        ray.along(self.near)
    }
}

/// A trait that represents any type that can be intersected by a Ray.
pub trait Intersect {
    fn intersect(ray: &Ray) -> Option<Hit>;
}

/// A trait that represents any type that is a scene object, and can thus be viewed in the final render.
pub trait SceneObject: Intersect {
    
}
