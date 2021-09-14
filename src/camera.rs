use crate::math::{Matrix, Vector3};

/// A Camera object. Represents a viewable area that a scene can be rendered to.
#[derive(Clone, Debug)]
pub struct Camera {
    pub vw: i32,
    pub vh: i32,
    pub origin: Vector3,
    pub yaw: f64,
    pub pitch: f64,
    pub fov: f64,
    pub chf: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            vw: 300,
            vh: 200,
            origin: Vector3::default(),
            yaw: 0.,
            pitch: 0.,
            fov: 60.,
            chf: Self::chf(60.),
        }
    }
}

impl Camera {
    /// Calculate the chf for an FOV.
    fn chf(fov: f64) -> f64 {
        ((90. - fov * 0.5) * 0.017453).tan()
    }

    /// Calculate the Vector3 direction for a given screen point.
    pub fn direction_at(&self, x: f64, y: f64) -> Vector3 {
        (Matrix::from_forward(self.direction_fov(x, y))
            * Matrix::from_euler_xyz(self.pitch, self.yaw, 0.))
        .forward()
    }

    /// Calculate the direction of a pixel on the camera based on the FOV, in camera space.
    pub fn direction_fov(&self, x: f64, y: f64) -> Vector3 {
        let nx = x - self.vw as f64 * 0.5;
        let ny = y - self.vh as f64 * 0.5;
        let z = self.vh as f64 * 0.5 * self.chf;
        Vector3::new(z, nx, -ny).normalize()
    }
}
