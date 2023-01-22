//! Virtual Reality interfacing

use crate::Transform;
use cimvr_engine_interface::prelude::*;
use serde::{Deserialize, Serialize};

/// VR update message
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VrUpdate {
    /// View for left eye (may lag behind view in shaders by a small duration!)
    pub view_left: Transform,
    /// View for right eye (may lag behind view in shaders by a small duration!)
    pub view_right: Transform,

    /// Projection parameters for left eye
    pub fov_left: VrFov,
    /// Projection parameters for right eye
    pub fov_right: VrFov,

    /// Left hand aim
    pub aim_left: Option<Transform>,
    /// Right hand aim
    pub aim_right: Option<Transform>,

    /// Left hand grip
    pub grip_left: Option<Transform>,
    /// Right hand grip
    pub grip_right: Option<Transform>,
    // /// All VR events
    // pub events: Vec<VrEvent>,
}

/// Field of view of OpenXR camera
/// Matches https://registry.khronos.org/OpenXR/specs/1.0/html/xrspec.html#XrFovf
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VrFov {
    /// Angle of the left side of the field of view. For a symmetric field of view this value is negative.
    pub angle_left: f32,
    /// Angle of the right side of the field of view.
    pub angle_right: f32,
    /// Angle of the top part of the field of view.
    pub angle_up: f32,
    /// Angle of the bottom part of the field of view. For a symmetric field of view this value is negative.
    pub angle_down: f32,
}

/*
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum VrEvent {
    // TODO: Events from controllers!
}
*/

impl Message for VrUpdate {
    const CHANNEL: ChannelId = ChannelId {
        id: 0x1337_BEA7,
        locality: Locality::Local,
    };
}
