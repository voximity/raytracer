use crate::math::{Matrix, Vector3};

/// A Camera object. Represents a viewable area that a scene can be rendered to.
#[derive(Clone, Debug)]
pub struct Camera {
    /// The viewport width.
    pub vw: i32,

    /// The viewport height.
    pub vh: i32,

    /// The origin of the camera.
    pub origin: Vector3,

    /// The yaw of the camera's rotation.
    pub yaw: f64,

    /// The pitch of the camera's rotation.
    pub pitch: f64,

    /// The camera's vertical FOV in degrees. Set using
    /// [`set_fov`](Self::set_fov)
    pub fov: f64,

    /// A precomputed value used when determining ray direction from pixel. Do not set.
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

    pub fn set_fov(&mut self, fov: f64) {
        self.fov = fov;
        self.chf = Self::chf(fov);
    }

    /// Calculate the Vector3 direction for a given screen point.
    pub fn direction_at(&self, x: f64, y: f64) -> Vector3 {
        (Matrix::from_forward(self.direction_fov(x, y))
            * Matrix::from_euler_xyz(-self.pitch, self.yaw, 0.))
        .forward()
    }

    /// Calculate the direction of a pixel on the camera based on the FOV, in camera space.
    pub fn direction_fov(&self, x: f64, y: f64) -> Vector3 {
        let nx = x - self.vw as f64 * 0.5;
        let ny = y - self.vh as f64 * 0.5;
        let z = self.vh as f64 * 0.5 * self.chf;
        Vector3::new(nx, -ny, -z).normalize()
    }
}
