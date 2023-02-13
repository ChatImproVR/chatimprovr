use anyhow::{format_err, Result};
use cimvr_common::gamepad::{Axis, Button, GamepadState, Gamepad};
use gilrs::{GamepadId, Gilrs};
use log::info;

// TODO: What happens on disconnect?

pub struct GamepadPlugin {
    gilrs: Gilrs,
    gamepads: Vec<GamepadId>,
}

impl GamepadPlugin {
    pub fn new() -> Result<Self> {
        let gilrs = Gilrs::new().map_err(|e| format_err!("{}", e))?;
        let mut gamepads = vec![];

        for (id, pad) in gilrs.gamepads() {
            gamepads.push(id);
            info!("Pad {} connected as {}", pad.name(), id);
        }

        Ok(GamepadPlugin { gilrs, gamepads })
    }

    pub fn update(&mut self) -> GamepadState {
        let mut state = vec![];

        while self.gilrs.next_event().is_some() {}

        for &id in &self.gamepads {
            let mut pad_state = Gamepad::new();
            let pad = self.gilrs.gamepad(id);
            for button in Button::BUTTONS {
                let data = pad.button_data(button_to_gilrs(button));
                let data = match data {
                    Some(d) => d.is_pressed(),
                    None => false,
                };
                pad_state.buttons.insert(button, data);
            }
                
            for axis in Axis::AXES {
                let data = pad.axis_data(axis_to_gilrs(axis));
                let data = match data {
                    Some(d) => d.value(),
                    None => 0.0,
                };
                pad_state.axes.insert(axis, data);
            }

            state.push(pad_state);
        }

        GamepadState(state)
    }
}

fn button_to_gilrs(button: Button) -> gilrs::Button {
    match button {
        Button::South => gilrs::Button::South,
        Button::East => gilrs::Button::East,
        Button::North => gilrs::Button::North,
        Button::West => gilrs::Button::West,
        Button::C => gilrs::Button::C,
        Button::Z => gilrs::Button::Z,
        Button::LeftTrigger => gilrs::Button::LeftTrigger,
        Button::LeftTrigger2 => gilrs::Button::LeftTrigger2,
        Button::RightTrigger => gilrs::Button::RightTrigger,
        Button::RightTrigger2 => gilrs::Button::RightTrigger2,
        Button::Select => gilrs::Button::Select,
        Button::Start => gilrs::Button::Start,
        Button::Mode => gilrs::Button::Mode,
        Button::LeftThumb => gilrs::Button::LeftThumb,
        Button::RightThumb => gilrs::Button::RightThumb,
        Button::DPadUp => gilrs::Button::DPadUp,
        Button::DPadDown => gilrs::Button::DPadDown,
        Button::DPadLeft => gilrs::Button::DPadLeft,
        Button::DPadRight => gilrs::Button::DPadRight,
    }
}

fn axis_to_gilrs(axis: Axis) -> gilrs::Axis {
    match axis {
        Axis::LeftStickX => gilrs::Axis::LeftStickX,
        Axis::LeftStickY => gilrs::Axis::LeftStickY,
        Axis::LeftZ => gilrs::Axis::LeftZ,
        Axis::RightStickX => gilrs::Axis::RightStickX,
        Axis::RightStickY => gilrs::Axis::RightStickY,
        Axis::RightZ => gilrs::Axis::RightZ,
    }
}
