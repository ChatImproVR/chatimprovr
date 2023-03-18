use cimvr_engine_interface::prelude::EngineIo;
use std::collections::HashSet;

use crate::desktop::{
    ElementState, InputEvent, KeyCode, KeyboardEvent, ModifiersState, MouseButton, MouseEvent,
    WindowEvent,
};

/// A helper struct for handling input events. This is a wrapper around the `InputEvent` enum.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct InputHelper {
    /// Keeps track of keys that are currently being pressed. A pressed key will be in this set.
    pub pressed_keys: HashSet<KeyCode>,
    /// Modifiers don't keep track of state, going up and down is treated like a boolean (on/off).
    pub modifiers_state: ModifiersState,
    /// Holds information about everything the mouse can do. See
    /// [`MouseState`](cimvr_common::utils::input_helper::MouseState).
    mouse_state: MouseState,
    /// Help keep track of window events like when the window gets resized.
    pub window_state: WindowEvent,
}

/// MouseState struct exists for the InputHelper the utilize and capture mouse information.
#[derive(Debug, Default, Clone, PartialEq)]
struct MouseState {
    /// Position of mouse in (X,Y) axis.
    pub position: (f32, f32),
    /// Scroll can be give as both Horizontal and Vertical scroll.
    pub scroll: (f32, f32),
    /// We don't need to have modifiers here since this can be checked through the InputHelper
    /// struct
    pub buttons: HashSet<MouseButton>, // When handling the events, we can check for the element
    /// state since that's an input event.
    pub in_window: bool,
}

impl InputHelper {
    /// This will set the InputHelper to hopefully reasonable defaults.
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
            window_state: WindowEvent::default(),
        }
    }

    /// Used to populate the InputHelper by grabbing the InputEvents from the inbox.
    ///
    /// # Arguments
    ///
    /// * `io` - A mutable reference to the engine IO manager.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() { handle_input_events()}
    /// # fn do_stuff() {} // placeholder for doctest.
    ///
    /// struct ClientState {
    ///     input_helper: InputHelper
    /// }
    ///
    /// fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
    ///     self.input_helper.handle_input_events(io);
    ///     // ..snip
    ///     for frame in io.inbox::<FrameTime>() {
    ///         // DeltaTime checks for input
    ///         if self.input_helper.key_down(KeyCode::W) {
    ///             do_stuff();
    ///         }
    ///     }
    /// }
    /// ```
    pub fn handle_input_events(&mut self, io: &mut EngineIo) {
        for event in io.inbox::<InputEvent>() {
            self.update(&event);
        }
    }

    fn update(&mut self, input_event: &InputEvent) {
        match input_event {
            InputEvent::Keyboard(keyboard_event) => self.handle_keyboard_event(keyboard_event),
            InputEvent::Mouse(mouse_event) => self.handle_mouse_event(mouse_event),
            InputEvent::Window(window_event) => self.handle_window_event(window_event),
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
                // Just replace the old modifier state.
                self.modifiers_state = *modifiers_state;
            }
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: &MouseEvent) {
        match mouse_event {
            MouseEvent::Moved(x, y) => self.mouse_state.position = (*x, *y),
            // TODO: Check if MouseEvent::Scrolled is (horizontal, vertizal), or (vertical,
            // horizontal)
            MouseEvent::Scrolled(hor, vert) => self.mouse_state.scroll = (*hor, *vert),
            MouseEvent::Entered => self.mouse_state.in_window = true,
            MouseEvent::Exited => self.mouse_state.in_window = false,
            MouseEvent::Clicked(button, element_state, modifiers_state) => {
                self.modifiers_state = *modifiers_state;
                match element_state {
                    ElementState::Pressed => {
                        self.mouse_state.buttons.insert(*button);
                    }
                    ElementState::Released => {
                        self.mouse_state.buttons.remove(button);
                    }
                }
            }
        }
    }

    fn handle_window_event(&mut self, window_event: &WindowEvent) {
        // To prevent the window event from getting needlessly checked *every* frame let's add a
        // quick check to see if it's the same.
        // TODO: Somehow check if this is better than matching every frame.
        if self.window_state.eq(window_event) {
            return;
        } else {
            match window_event {
                WindowEvent::Resized { width, height } => {
                    self.window_state = WindowEvent::Resized {
                        height: *height,
                        width: *width,
                    }
                }
            }
        }
    }
}
