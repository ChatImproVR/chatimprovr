use cimvr_engine_interface::prelude::EngineIo;
use std::collections::HashSet;

use crate::desktop::{
    ElementState, InputEvent, KeyCode, KeyboardEvent, ModifiersState, MouseButton,
};

/// A helper struct for handling input events. This is a wrapper around the `InputEvent` enum.
#[derive(Debug, Default, Clone, PartialEq)]
struct InputHelper {
    /// Keeps track of keys that are currently being pressed. A pressed key will be in this set.
    pub pressed_keys: HashSet<KeyCode>,
    /// Modifiers don't keep track of state, going up and down is treated like a boolean (on/off).
    pub modifiers_state: ModifiersState,
    pub mouse_state: MouseState,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct MouseState {
    pub position: (f32, f32),
    pub scroll: (f32, f32),
    pub buttons: HashSet<MouseButton>, // When handling the events, we can check for the element
    // We don't
    // state since that's an input event.
    pub in_window: bool,
}

pub struct MouseButtonState {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub other: HashSet<u16>,
}

impl InputHelper {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            modifiers_state: ModifiersState::default(),
            mouse_state: MouseState {
                position: (0.0, 0.0),
                scroll: (0.0, 0.0),
                buttons: HashSet::new(),
                in_window: false,
            },
        }
    }

    pub fn handle_input_events(&mut self, io: &mut EngineIo) {
        for event in io.inbox::<InputEvent>() {
            self.update(&event);
        }
    }

    fn update(&mut self, input_event: &InputEvent) {
        match input_event {
            InputEvent::Keyboard(keyboard_event) => self.handle_keyboard_event(keyboard_event),
            _ => {}
        }
    }

    fn handle_keyboard_event(&mut self, keyboard_event: &KeyboardEvent) {
        match keyboard_event {
            KeyboardEvent::Key { key, state } => match state {
                ElementState::Pressed => {
                    self.pressed_keys.insert(*key);
                }
                ElementState::Released => {
                    self.pressed_keys.remove(key);
                }
            },
            KeyboardEvent::Modifiers(modifiers_state) => {
                self.modifiers_state = *modifiers_state;
            }
        }
    }
}
