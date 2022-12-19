use cimvr_engine_interface::prelude::*;
pub use nalgebra;
use nalgebra::{Point3, UnitQuaternion};
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

/// Simple string message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StringMessage(pub String);

impl Message for StringMessage {
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
