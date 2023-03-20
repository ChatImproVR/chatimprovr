//! Virtual Reality interfacing
use crate::Transform;
use cimvr_engine_interface::{pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

/// VR update message
///
/// NOTE: All coordinates are relative to the "Floor" VR space.
/// That is, (0, 0, 0) is always the middle of the floor in standing mode, regardless of where the camera is
#[derive(Message, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[locality("Local")]
pub struct VrUpdate {
    /// State of the headset
    pub headset: HeadsetState,
    /// State of the left controller, and events
    pub left_controller: ControllerState,
    /// State of the right controller, and events
    pub right_controller: ControllerState,
}

/// State of the headset (including view and projection)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct HeadsetState {
    /// Left eye
    pub left: ViewState,
    /// Right eye
    pub right: ViewState,
}

/// State of a particular view (eye)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ViewState {
    /// Position and orientation of this eye
    pub transf: Transform,
    /// Projection for this eye
    pub proj: VrFov,
}

/// State of a particular controller
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ControllerState {
    /// Aim transform of this controller (for e.g. a gun). May be None if no controller is connected
    pub aim: Option<Transform>,
    /// Grip transform of this controller (for e.g. a cup). May be None if no controller is connected
    pub grip: Option<Transform>,
    // /// Events captured during this update
    pub events: Vec<ControllerEvent>,
}

/// Events produced by a controller
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ControllerEvent {
    /// The menu button state has updated
    Menu(ElementState),
    /// The select button state has updated
    Trigger(ElementState),
}

/// State of a button
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ElementState {
    Pressed,
    Released,
}

/// Field of view of OpenXR camera
/// Matches <https://registry.khronos.org/OpenXR/specs/1.0/html/xrspec.html#XrFovf>
#[derive(Serialize, Deserialize, Copy, Debug, Clone, PartialEq)]
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

impl From<bool> for ElementState {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Pressed,
            false => Self::Released,
        }
    }
}
