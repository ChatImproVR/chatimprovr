#![cfg(not(doctest))]
//! # Common
//! This crate is intended to facilitate communication with the specific server and client
//! implementations provided alongside ChatimproVR. This library is always used in conjunction with
//! `engine_interface`.
use cimvr_engine_interface::{pkg_namespace, prelude::*};
pub use nalgebra;
use nalgebra::{Isometry3, Matrix4, Point3, Translation3, UnitQuaternion};
use serde::{Deserialize, Serialize};

pub mod desktop;
pub mod gamepad;
pub mod render;
pub mod ui;
pub mod vr;

/// Component representing position and orientation
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    /// Position
    pub pos: Point3<f32>,
    /// Orientation (Rotation)
    pub orient: UnitQuaternion<f32>,
}

impl Component for Transform {
    const ID: ComponentIdStatic = ComponentIdStatic {
        // steakhouse
        id: pkg_namespace!("Transform"),
        size: 44,
    };
}

/// Frame information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FrameTime {
    /// Delta time, in seconds
    pub delta: f32,
    /// Time since engine start, in seconds
    pub time: f32,
}

impl Message for FrameTime {
    const CHANNEL: ChannelIdStatic = ChannelIdStatic {
        // That's what I've been waitin for, that's what it's all about! Wahoo!
        id: pkg_namespace!("FrameTime"),
        locality: Locality::Local,
    };
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    /// Turn it into a Matrix;
    /// Represent the transformation as a linear transformation of homogeneous coordinates.
    pub fn to_homogeneous(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&self.pos.coords) * self.orient.to_homogeneous()
    }

    /// Construct a view matrix from this transform; it's actually the inverse of to_homogeneous()!
    pub fn view(&self) -> Matrix4<f32> {
        // Invert this quaternion, orienting the world into NDC space
        // Represent the rotation in homogeneous coordinates
        let rotation = self.orient.inverse().to_homogeneous();

        // Invert this translation, translating the world into NDC space
        let translation = Matrix4::new_translation(&-self.pos.coords);

        // Compose the view
        rotation * translation
    }

    /// The identity transformation
    pub fn identity() -> Self {
        Self {
            pos: Point3::origin(),
            orient: UnitQuaternion::identity(),
        }
    }

    pub fn with_position(mut self, pos: Point3<f32>) -> Self {
        self.pos = pos;
        self
    }

    pub fn with_rotation(mut self, orient: UnitQuaternion<f32>) -> Self {
        self.orient = orient;
        self
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

impl Into<Isometry3<f32>> for Transform {
    fn into(self) -> Isometry3<f32> {
        Isometry3 {
            rotation: self.orient,
            translation: Translation3::from(self.pos),
        }
    }
}

impl From<Isometry3<f32>> for Transform {
    fn from(value: Isometry3<f32>) -> Self {
        Self {
            pos: value.translation.vector.into(),
            orient: value.rotation,
        }
    }
}
