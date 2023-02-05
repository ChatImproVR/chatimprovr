//! # Common
//! This crate is intended to facilitate communication with the specific server and client
//! implementations provided alongside ChatimproVR. This library is always used in conjunction with
//! `engine_interface`.
use cimvr_engine_interface::prelude::*;
pub use nalgebra;
use nalgebra::{Matrix4, Point3, UnitQuaternion};
use serde::{Deserialize, Serialize};

pub mod desktop;
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
    const ID: ComponentId = ComponentId {
        // steakhouse
        id: 0xDEAD_BEEF_CAFE,
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
    const CHANNEL: ChannelId = ChannelId {
        // That's what I've been waitin for, that's what it's all about! Wahoo!
        id: 0x0000000_EEEAAA_BABEEE,
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
}
