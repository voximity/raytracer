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
    pub fn right_vector(self) -> Vector3 {
        Vector3::new(self.m00, self.m10, self.m20)
    }

    /// Get the up vector of this matrix.
    pub fn up_vector(self) -> Vector3 {
        Vector3::new(self.m01, self.m11, self.m21)
    }

    /// Get the forward vector of this matrix.
    pub fn forward_vector(self) -> Vector3 {
        Vector3::new(-self.m02, -self.m12, -self.m22)
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
