//! Virtual Reality interfacing

use crate::Transform;
use cimvr_engine_interface::prelude::*;
use serde::{Deserialize, Serialize};

/// VR update message
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VrUpdate {
    /// View for left eye (may lag behind view in shaders by a small duration!)
    pub left_view: Transform,
    /// View for right eye (may lag behind view in shaders by a small duration!)
    pub right_view: Transform,

    /// Projection parameters for left eye
    pub left_fov: VrFov,
    /// Projection parameters for right eye
    pub right_fov: VrFov,
    /*
    /// Right hand transform
    pub right_hand: Transform,
    /// Left hand transform
    pub left_hand: Transform,

    /// All VR events
    pub events: Vec<VrEvent>,
    */
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum VrEvent {
    // TODO: Events from controllers!
}

impl Message for VrUpdate {
    const CHANNEL: ChannelId = ChannelId {
        id: 0x1337_BEA7,
        locality: Locality::Local,
    };
}
