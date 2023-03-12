#![cfg(not(doctest))]
//! # Common
//! This crate is intended to facilitate communication with the specific server and client
//! implementations provided alongside ChatimproVR. This library is always used in conjunction with
//! `engine_interface`.
use std::ops::Mul;

use cimvr_engine_interface::{pkg_namespace, prelude::*};
pub use glam;
use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

pub mod desktop;
pub mod gamepad;
pub mod render;
pub mod ui;
pub mod utils;
pub mod vr;

/// # Component representing position and orientation
///
/// Represents a rotation, followed by a translation.
/// Composable through the multiplication (`*`) operator.
/// Features `From`/`Into` for `Isometry3<f32>`.
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    /// Position
    pub pos: Vec3,
    /// Orientation (Rotation)
    pub orient: Quat,
}

impl Component for Transform {
    const ID: &'static str = pkg_namespace!("Transform");
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

    pub fn inverse(self) -> Self {
        let orient = self.orient.inverse();
        Self {
            orient,
            pos: self.orient * -self.pos,
        }
    }
}

/// A generic handle type, which is integer sized but represents a namespace
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct GenericHandle(u128);

impl GenericHandle {
    /// Create a handle from the given name. Hashes the string
    pub const fn new(name: &str) -> Self {
        Self(const_hash(name))
    }

    /// Create a handle within this namespace, indexed by `i`.
    /// Note that this is a deterministic function!
    pub fn index(self, i: u128) -> Self {
        Self(self.0.wrapping_add(i))
    }
}

/// A pretty bad hash function. Made constant so you can have things like
/// ```rust
/// use cimvr_engine_interface::{pkg_namespace, prelude::*};
/// use cimvr_common::render::RenderHandle;
/// const CUBE_HANDLE: RenderHandle = RenderHandle::new(pkg_namespace!("Cube"));
/// ```
const fn const_hash(s: &str) -> u128 {
    const C: u128 = 31;
    let mut hash: u128 = 0;
    let mut i = 0;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        let b = bytes[i] as u128;
        hash = hash.wrapping_mul(C).wrapping_add(b);
        i += 1;
    }
    hash
}

/// Creates a handle type wrapping a GenericHandle. For example:
/// ```rust
/// struct MyHandle(GenericHandle);
/// make_handle!(MyHandle);
/// ```
#[macro_export]
macro_rules! make_handle {
    ($name:ident) => {
        impl $name {
            /// Create a handle from the given name. Hashes the string
            pub const fn new(name: &str) -> Self {
                Self(GenericHandle::new(name))
            }

            /// Create a handle within this namespace, indexed by `i`.
            /// Note that this is a deterministic function!
            pub fn index(self, i: u128) -> Self {
                Self(self.0.index(i))
            }
        }
    };
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

impl Default for GenericHandle {
    fn default() -> Self {
        Self(0xBAD_BAD_BAD_BAD_BAD_BAD_BAD_BAD_BAD_BAD)
    }
}
