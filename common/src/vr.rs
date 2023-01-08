//! Virtual Reality interfacing

use crate::Transform;
use serde::{Deserialize, Serialize};

/// VR update message
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VrUpdate {
    /// View for left eye (may lag behind view in shaders by a small duration!)
    pub left_view: Transform,
    /// View for right eye (may lag behind view in shaders by a small duration!)
    pub right_view: Transform,

    /// Projection parameters for left eye
    pub left_proj: VrProjection,
    /// Projection parameters for right eye
    pub right_proj: VrProjection,

    /// Right hand transform
    pub right_hand: Transform,
    /// Left hand transform
    pub left_hand: Transform,

    /// All VR events
    pub events: Vec<VrEvent>,
}

/// Matches https://registry.khronos.org/OpenXR/specs/1.0/html/xrspec.html#XrFovf
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VrProjection {
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
