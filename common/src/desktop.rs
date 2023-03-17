//! Types useful for interfacing with the Host's mouse and keyboard
use cimvr_engine_interface::{pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

// TODO: Gamepad support!
// TODO: Touchscreen support!

/// Basic input events
#[derive(Message, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[locality("Local")]
pub enum InputEvent {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
    Window(WindowEvent),
}

impl InputEvent {
    fn get_key_events(&self) -> Option<&KeyboardEvent> {
        match self {
            Self::Keyboard(key_event) => Some(key_event),
            _ => None,
        }
    }

    /// Attempts to get the keycode and element state of an InputEvent if one exists.
    ///
    /// Useful if you simply want to get a key and whether or not its being pressed.
    ///
    /// # Examples
    ///
    /// ```
    /// for (key, state) in io.inbox::<InputEvent>.filter_map(|e| x.get_keyboard()) {
    ///     let is_pressed = state == ElementState::Pressed;
    ///
    ///     match key {
    ///         KeyCode::W => { .. },
    ///         ..snip..
    ///         }
    ///     }
    /// }
    /// ```
    pub fn get_keyboard(&self) -> Option<(KeyCode, ElementState)> {
        match self.get_key_events() {
            Some(KeyboardEvent::Key { key, state }) => Some((*key, *state)),
            _ => None,
        }
    }

    /// Retrieves `ModifiersState` from an `InputEvent` if it exists.
    ///
    /// Useful if you want to get the current modifier.
    pub fn get_modifier_state(&self) -> Option<ModifiersState> {
        match self.get_key_events() {
            Some(KeyboardEvent::Modifiers(m)) => Some(*m),
            _ => None,
        }
    }

    pub fn get_mouse(&self) -> Option<&MouseEvent> {
        match self {
            Self::Mouse(mouse_event) => Some(mouse_event),
            _ => None,
        }
    }
    pub fn get_window(&self) -> Option<&WindowEvent> {
        match self {
            Self::Window(window_event) => Some(window_event),
            _ => None,
        }
    }
}

/// Basic mouse events
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MouseEvent {
    Moved(f32, f32),
    Scrolled(f32, f32),
    Entered,
    Exited,
    Clicked(MouseButton, ElementState, ModifiersState),
}

/// Basic window events
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum WindowEvent {
    /// Window's size in pixels changed
    Resized { width: u32, height: u32 },
}

/// Keyboard events
#[derive(Serialize, Deserialize, Hash, Copy, Debug, Clone, PartialEq, Eq)]
pub enum KeyboardEvent {
    Key {
        /// Key used
        key: KeyCode,
        /// State of the key
        state: ElementState,
    },
    Modifiers(ModifiersState),
}

/// Keyboard Modifier states
#[derive(Serialize, Deserialize, Hash, Copy, Debug, Clone, PartialEq, Eq, Default)]
pub struct ModifiersState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

/// Describes a button of a mouse controller.
#[derive(Serialize, Deserialize, Hash, Copy, Debug, Clone, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

/// Describes the input state of a key.
#[derive(Serialize, Deserialize, Hash, Copy, Debug, Clone, PartialEq, Eq)]
pub enum ElementState {
    Pressed,
    Released,
}

/// KeyCode; matches winit's keycode enum.
#[derive(Serialize, Deserialize, Hash, Copy, Debug, Clone, PartialEq, Eq)]
pub enum KeyCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    Back,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    // also called "Next"
    NavigateForward,
    // also called "Prior"
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}
