use std::collections::HashSet;
use cimvr_engine_interface::prelude::EngineIo;

use crate::desktop::{KeyCode, ModifiersState, MouseButton, InputEvent, KeyboardEvent, ElementState};

/// A helper struct for handling input events. This is a wrapper around the `InputEvent` enum.
#[derive(Debug, Default, Clone, PartialEq)]
struct InputHelper {
    pub keys: HashSet<KeyCode>,
    pub pressed_modifiers: ModifiersState,
    pub mouse_state: MouseState
}

#[derive(Debug, Default, Clone, PartialEq)]
struct MouseState {
    pub position: (f32, f32),
    pub scroll: (f32, f32),
    pub buttons: HashSet<MouseButton>,
    // pub modifiers: ModifiersState,
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
            keys: HashSet::new(),
            pressed_modifiers: ModifiersState::default(),
            mouse_state: MouseState {
                position: (0.0, 0.0),
                scroll: (0.0, 0.0),
                buttons: HashSet::new(),
                // modifiers: ModifiersState::default(),
                in_window: false,
            }
        }
    }

    pub fn handle_input_events(&mut self, io: &mut EngineIo) {
        for event in io.inbox::<InputEvent>() {
            self.update(&event);
        }
    }
    
    fn update(&mut self, input_event: &InputEvent){
        match input_event {
            InputEvent::Keyboard(keyboard_event) => self.handle_keyboard_event(keyboard_event),
            _ => {}
        }
    }

    fn handle_keyboard_event(&mut self, keyboard_event: &KeyboardEvent){
        match keyboard_event {
            KeyboardEvent::Key { key, state } => {
                match state {
                    ElementState::Pressed => {
                        self.keys.insert(*key);
                    }
                    ElementState::Released => {
                        self.keys.remove(key);
                    }
                }
            }
            KeyboardEvent::Modifiers(modifiers_state) => {
                self.pressed_modifiers = *modifiers_state;
            }
        }
    }
}