use cimvr_engine_interface::prelude::*;
pub use nalgebra;
use nalgebra::{Matrix4, Point3, UnitQuaternion};
use serde::{Deserialize, Serialize};

pub mod input;
pub mod render;

/// Component representing positino and orientation
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
        size: 30,
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
        Self {
            pos: Point3::origin(),
            orient: UnitQuaternion::identity(),
        }
    }
}

impl Transform {
    /// Turn it into a Matrix;
    /// Represent the transformation as a linear transformation of homogeneous coordinates.
    pub fn to_homogeneous(&self) -> Matrix4<f32> {
        Matrix4::new_translation(&self.pos.coords) * self.orient.to_homogeneous()
    }
}
