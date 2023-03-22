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
    held_keys: HashSet<KeyCode>,
    /// Modifiers don't keep track of state, going up and down is treated like a boolean (on/off).
    modifiers_state: ModifiersState,
    /// Holds information about everything the mouse can do. See
    /// [`MouseState`](cimvr_common::utils::input_helper::MouseState).
    mouse_state: MouseState,
    /// Help keep track of window events like when the window gets resized.
    window_state: WindowEvent,
    /// This field gets cleared every update step.
    pressed_keys: HashSet<KeyCode>,
    /// This field also gets cleared every update step.
    released_keys: HashSet<KeyCode>,
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
    pub held_buttons: HashSet<MouseButton>, // When handling the events, we can check for the element
    /// This field gets cleared every update step.
    pub pressed_buttons: HashSet<MouseButton>,
    /// This field also gets cleared every update step.
    pub released_buttons: HashSet<MouseButton>,
    /// state since that's an input event.
    pub in_window: bool,

    pub prev_pos: (f32, f32),
}

impl InputHelper {
    /// This will set the InputHelper to hopefully reasonable defaults.
    pub fn new() -> Self {
        Self {
            held_keys: HashSet::new(),
            modifiers_state: ModifiersState::default(),
            mouse_state: MouseState {
                position: (0.0, 0.0),
                scroll: (0.0, 0.0),
                held_buttons: HashSet::new(),
                pressed_buttons: HashSet::new(),
                released_buttons: HashSet::new(),
                in_window: false,
                prev_pos: (0., 0.),
            },
            window_state: WindowEvent::default(),
            pressed_keys: HashSet::new(),
            released_keys: HashSet::new(),
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
        self.pressed_keys.clear();
        self.released_keys.clear();
        self.mouse_state.pressed_buttons.clear();
        self.mouse_state.released_buttons.clear();
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
                    self.held_keys.insert(*key);
                }
                ElementState::Released => {
                    self.released_keys.insert(*key);
                    self.held_keys.remove(key);
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
            MouseEvent::Moved(x, y) => {
                self.mouse_state.prev_pos = self.mouse_state.position;
                self.mouse_state.position = (*x, *y)
            }
            MouseEvent::Scrolled(hor, vert) => self.mouse_state.scroll = (*hor, *vert),
            MouseEvent::Entered => self.mouse_state.in_window = true,
            MouseEvent::Exited => self.mouse_state.in_window = false,
            MouseEvent::Clicked(button, element_state, modifiers_state) => {
                self.modifiers_state = *modifiers_state;
                match element_state {
                    ElementState::Pressed => {
                        self.mouse_state.pressed_buttons.insert(*button);
                        self.mouse_state.held_buttons.insert(*button);
                    }
                    ElementState::Released => {
                        self.mouse_state.released_buttons.insert(*button);
                        self.mouse_state.held_buttons.remove(button);
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

    // --- Public API ---
    // Spec should include
    // key press api
    //
    // key_down(&self, keycode) -> bool
    // key_up(&self, keycode) -> bool
    pub fn key_held(&self, key: KeyCode) -> bool {
        return self.held_keys.contains(&key);
    }
    /// Checks if they key is pressed during that frame step. If the user is still holding the
    /// button pressed on the next frame step `key_pressed` will return false.
    pub fn key_pressed(&self, key: KeyCode) -> bool {
        return self.pressed_keys.contains(&key);
    }
    /// Checks if they key was released during that frame step. Subsequent calls to `key_released`
    /// will return false unless the key has been released again during that frame step.
    pub fn key_released(&self, key: KeyCode) -> bool {
        return self.released_keys.contains(&key);
    }

    // modifiers
    // held_alt(&self) -> bool
    // held_shift(&self) -> bool
    // held_control(&self) -> bool
    pub fn held_shift(&self) -> bool {
        return self.modifiers_state.alt;
    }
    pub fn held_ctrl(&self) -> bool {
        return self.modifiers_state.ctrl;
    }
    pub fn held_alt(&self) -> bool {
        return self.modifiers_state.alt;
    }
    pub fn held_logo(&self) -> bool {
        return self.modifiers_state.logo;
    }
    // mouse api
    // mouse_pressed(&self) -> bool
    // mouse_released(&self) -> bool
    //
    // TBD how we want to do this.
    // mousewheel_scroll_diff(&self) -> f32
    // mouse_pos(&self) -> Option<(f32, f32)>
    // mouse_pos_diff(&self) -> (f32,f32)
    pub fn mouse_held(&self, mouse_button: MouseButton) -> bool {
        return self.mouse_state.held_buttons.contains(&mouse_button);
    }

    pub fn mouse_diff(&self) -> (f32, f32) {
        let (x1, y1) = self.mouse_state.position;
        let (x2, y2) = self.mouse_state.prev_pos;
        let diff = (x1 - x2, y1 - y2);
        diff
    }
    /// Checks if the mouse has been pressed. Note that this only triggers when the key is pressed.
    /// An example for this would be if the user pressed the mouse button on frame one,
    /// `mouse_pressed` would return true. However, if the user is still holding the button after
    /// frame one `mouse_pressed` will return false.
    pub fn mouse_pressed(&self, mouse_button: MouseButton) -> bool {
        return self.mouse_state.pressed_buttons.contains(&mouse_button);
    }

    /// Checks if the mouse has been released. Note that this only triggers when the key is
    /// released. An example for this would be if the user released the mouse button on frame one,
    /// `mouse_released` would return true. If you were to check for the mouse being released after
    /// frame one, `mouse_released` would return false.
    pub fn mouse_released(&self, mouse_button: MouseButton) -> bool {
        return self.mouse_state.released_buttons.contains(&mouse_button);
    }

    pub fn mouse_pos(&self) -> Option<(f32, f32)> {
        self.mouse_state
            .in_window
            .then(|| self.mouse_state.position)
    }

    pub fn mousewheel_scroll_diff(&self) -> Option<(f32, f32)> {
        self.mouse_state.in_window.then(|| self.mouse_state.scroll)
    }

    // Screen resize API
    // get_resolution(&self) -> Option<(u32, u32)>
    pub fn get_resolution(&self) -> (u32, u32) {
        match self.window_state {
            WindowEvent::Resized { width, height } => (width, height),
        }
    }
}
