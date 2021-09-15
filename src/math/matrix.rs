use std::ops::Mul;

use super::Vector3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub m00: f64,
    pub m01: f64,
    pub m02: f64,
    pub m10: f64,
    pub m11: f64,
    pub m12: f64,
    pub m20: f64,
    pub m21: f64,
    pub m22: f64,
}

impl Matrix {
    /// Create a new matrix from 3 coordinates in world space.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Matrix {
            x,
            y,
            z,
            ..Default::default()
        }
    }

    /// Create a new matrix from a forward vector.
    ///
    /// **Note:** This vector is expected to be normalized.
    pub fn from_forward(vec: Vector3) -> Self {
        let forward = vec;
        let right = Vector3::up().cross(vec);
        let up = forward.cross(right);

        Matrix {
            x: 0.,
            y: 0.,
            z: 0.,
            m00: right.x,
            m01: right.y,
            m02: right.z,
            m10: up.x,
            m11: up.y,
            m12: up.z,
            m20: -forward.x,
            m21: -forward.y,
            m22: -forward.z,
        }
    }

    #[rustfmt::skip]
    fn euler_matrices(x: f64, y: f64, z: f64) -> (Self, Self, Self) {
        (
            Matrix { x: 0., y: 0., z: 0., m00: 1., m01: 0., m02: 0., m10: 0., m11: x.cos(), m12: -x.sin(), m20: 0., m21: x.sin(), m22: x.cos() },
            Matrix { x: 0., y: 0., z: 0., m00: y.cos(), m01: 0., m02: y.sin(), m10: 0., m11: 1., m12: 0., m20: -y.sin(), m21: 0., m22: y.cos() },
            Matrix { x: 0., y: 0., z: 0., m00: z.cos(), m01: -z.sin(), m02: 0., m10: z.sin(), m11: z.cos(), m12: 0., m20: 0., m21: 0., m22: 1. },
        )
    }

    /// Create a new matrix from Euler angles applied in XYZ order.
    pub fn from_euler_xyz(x: f64, y: f64, z: f64) -> Self {
        let (a, b, c) = Self::euler_matrices(x, y, z);
        a * b * c
    }

    /// Create a new matrix from Euler angles applied in ZYX order.
    pub fn from_euler_zyx(x: f64, y: f64, z: f64) -> Self {
        let (a, b, c) = Self::euler_matrices(x, y, z);
        c * b * a
    }

    /// Get the components of this matrix.
    #[rustfmt::skip]
    pub fn components(self) -> [f64; 16] {
        [
            self.m00, self.m01, self.m02, self.x,
            self.m10, self.m11, self.m12, self.y,
            self.m20, self.m21, self.m22, self.z,
            0., 0., 0., 1.,
        ]
    }

    /// Get the rowed components of this matrix.
    #[rustfmt::skip]
    pub fn rowed_components(self) -> [[f64; 4]; 4] {
        [
            [self.m00, self.m01, self.m02, self.x],
            [self.m10, self.m11, self.m12, self.y],
            [self.m20, self.m21, self.m22, self.z],
            [0., 0., 0., 1.],
        ]
    }

    /// Get the right vector of this matrix.
    pub fn right(self) -> Vector3 {
        Vector3::new(self.m00, self.m01, self.m02)
    }

    /// Get the up vector of this matrix.
    pub fn up(self) -> Vector3 {
        Vector3::new(self.m10, self.m11, self.m12)
    }

    /// Get the forward vector of this matrix.
    pub fn forward(self) -> Vector3 {
        Vector3::new(-self.m20, -self.m21, -self.m22)
    }

    /// Get the position vector of this matrix.
    pub fn pos(self) -> Vector3 {
        Vector3::new(self.x, self.y, self.z)
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 0.,
            m00: 1.,
            m01: 0.,
            m02: 0.,
            m10: 0.,
            m11: 1.,
            m12: 0.,
            m20: 0.,
            m21: 0.,
            m22: 0.,
        }
    }
}

impl From<Vector3> for Matrix {
    fn from(vec: Vector3) -> Self {
        Self::new(vec.x, vec.y, vec.z)
    }
}

impl Mul for Matrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let a = self.rowed_components();
        let b = rhs.rowed_components();
        let mut o = [
            [0., 0., 0., 0.],
            [0., 0., 0., 0.],
            [0., 0., 0., 0.],
            [0., 0., 0., 0.],
        ];

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    o[i][j] += a[i][k] * b[k][j];
                }
            }
        }

        Self {
            x: o[0][3],
            y: o[1][3],
            z: o[2][3],
            m00: o[0][0],
            m01: o[0][1],
            m02: o[0][2],
            m10: o[1][0],
            m11: o[1][1],
            m12: o[1][2],
            m20: o[2][0],
            m21: o[2][1],
            m22: o[2][2],
        }
    }
}
