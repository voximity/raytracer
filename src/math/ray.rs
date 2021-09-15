use super::Vector3;

#[derive(Clone, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

impl Ray {
    /// Instantiate a new Ray. The direction is expected to be already normalized.
    pub fn new(origin: Vector3, direction: Vector3) -> Self {
        Self { origin, direction }
    }

    pub fn along(&self, t: f64) -> Vector3 {
        self.origin + self.direction * t
    }

    pub fn closest(&self, vec: Vector3) -> Vector3 {
        let ap = vec - self.origin;
        let ab = self.direction;
        self.along(ap.dot(ab) / ab.dot(ab))
    }

    /// Reflect this ray off of a position and a normal.
    pub fn reflect(&self, pos: Vector3, normal: Vector3) -> Ray {
        let dir = self.direction - normal * (2. * self.direction.dot(normal));
        Ray::new(pos, dir)
    }
}
