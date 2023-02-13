use std::collections::HashMap;

use cimvr_engine_interface::{pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

/// State of each gamepad
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GamepadState(pub Vec<Gamepad>);

impl Message for GamepadState {
    const CHANNEL: ChannelIdStatic = ChannelIdStatic {
        id: pkg_namespace!("GamepadState"),
        locality: Locality::Local,
    };
}

/// Entire state of one game pad
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Gamepad {
    pub buttons: HashMap<Button, bool>,
    pub axes: HashMap<Axis, f32>,
}

impl Gamepad {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Gamepad axis
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum Axis {
    LeftStickX,
    LeftStickY,
    LeftZ,
    RightStickX,
    RightStickY,
    RightZ,
}

/// Gamepad button
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum Button {
    South,
    East,
    North,
    West,
    C,
    Z,
    LeftTrigger,
    LeftTrigger2,
    RightTrigger,
    RightTrigger2,
    Select,
    Start,
    Mode,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

impl Button {
    pub const BUTTONS: [Self; 19] = [
        Self::South,
        Self::East,
        Self::North,
        Self::West,
        Self::C,
        Self::Z,
        Self::LeftTrigger,
        Self::LeftTrigger2,
        Self::RightTrigger,
        Self::RightTrigger2,
        Self::Select,
        Self::Start,
        Self::Mode,
        Self::LeftThumb,
        Self::RightThumb,
        Self::DPadUp,
        Self::DPadDown,
        Self::DPadLeft,
        Self::DPadRight,
    ];
}

impl Axis {
    pub const AXES: [Self; 6] = [
        Self::LeftStickX,
        Self::LeftStickY,
        Self::LeftZ,
        Self::RightStickX,
        Self::RightStickY,
        Self::RightZ,
    ];
}
