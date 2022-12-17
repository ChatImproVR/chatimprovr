use cimvr_engine_interface::prelude::*;
pub use nalgebra;
use nalgebra::{Point3, UnitQuaternion};
use serde::{Deserialize, Serialize};

/// A transform component on an entity
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: UnitQuaternion<f32>,
    //pub scale: Vector3<f32>,
}

impl Component for Transform {
    const ID: ComponentId = ComponentId {
        // steakhouse
        id: 0xDEAD_BEEF_CAFE,
        size: 30,
    };
}

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
            position: Point3::origin(),
            rotation: UnitQuaternion::identity(),
        }
    }
}
