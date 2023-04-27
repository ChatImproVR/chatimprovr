use cimvr_engine_interface::{pkg_namespace, prelude::*};
use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::ops::Mul;

/// # Component representing position and orientation
///
/// Represents a rotation, followed by a translation.
/// Composable through the multiplication (`*`) operator.
#[derive(Component, Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    /// Position
    pub pos: Vec3,
    /// Orientation (Rotation)
    pub orient: Quat,
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    /// Alias for Self::identity()
    pub fn new() -> Self {
        Self::identity()
    }

    /// Turn it into a Matrix;
    /// Represent the transformation as a linear transformation of homogeneous coordinates.
    pub fn to_homogeneous(&self) -> Mat4 {
        Mat4::from_translation(self.pos) * Mat4::from_quat(self.orient)
    }

    /// Construct a view matrix from this transform; it's actually the inverse of to_homogeneous()!
    pub fn view(&self) -> Mat4 {
        // Invert this quaternion, orienting the world into NDC space
        // Represent the rotation in homogeneous coordinates
        let rotation = Mat4::from_quat(self.orient.inverse());

        // Invert this translation, translating the world into NDC space
        let translation = Mat4::from_translation(-self.pos);

        // Compose the view
        rotation * translation
    }

    /// The identity transformation
    pub fn identity() -> Self {
        Self {
            pos: Vec3::ZERO,
            orient: Quat::IDENTITY,
        }
    }

    pub fn with_position(mut self, pos: Vec3) -> Self {
        self.pos = pos;
        self
    }

    pub fn with_rotation(mut self, orient: Quat) -> Self {
        self.orient = orient;
        self
    }

    /// Invert this transformation
    pub fn inverse(self) -> Self {
        let orient = self.orient.inverse();
        Self {
            orient,
            pos: orient * -self.pos,
        }
    }

    /// Interpolate between transforms
    pub fn lerp_slerp(&self, other: &Self, t: f32) -> Self {
        Self {
            pos: self.pos.lerp(other.pos, t),
            orient: self.orient.slerp(other.orient, t),
        }
    }
}

impl Into<Mat4> for Transform {
    fn into(self) -> Mat4 {
        Mat4::from_quat(self.orient) * Mat4::from_translation(self.pos)
    }
}

impl Mul for Transform {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            pos: self.pos + self.orient * rhs.pos,
            orient: self.orient * rhs.orient,
        }
    }
}
