use cimvr_common::desktop::*;
use cimvr_engine::Engine;

/// Input handler for Desktop platform
pub struct DesktopInputHandler {
    events: Vec<InputEvent>,
}

impl DesktopInputHandler {
    pub fn new() -> Self {
        Self { events: vec![] }
    }

    /// Returns the InputState which chronicles the events since the last call
    pub fn get_history(&mut self, engine: &mut Engine) {
        let drained_events = self.events.drain(..);
        for event in drained_events {
            engine.send(event);
        }
        // InputEvents(std::mem::take(&mut self.events))
    }

    /// Handle a Winit event
    pub fn handle_winit_event(&mut self, event: &glutin::event::WindowEvent) {
        match event {
            #[allow(deprecated)] // lol
            glutin::event::WindowEvent::KeyboardInput { input, .. } => {
                if let glutin::event::KeyboardInput {
                    state,
                    virtual_keycode: Some(key),
                    ..
                } = input
                {
                    let event = KeyboardEvent::Key {
                        key: map_keycode(*key),
                        state: map_elem_state(*state),
                    };
                    self.events.push(InputEvent::Keyboard(event));
                }
            }
            glutin::event::WindowEvent::ModifiersChanged(modifiers) => {
                self.events
                    .push(InputEvent::Keyboard(KeyboardEvent::Modifiers(
                        map_modifiers(*modifiers),
                    )))
            }
            glutin::event::WindowEvent::CursorMoved { position, .. } => {
                let event = MouseEvent::Moved(position.x as f32, position.y as f32);
                self.events.push(InputEvent::Mouse(event));
            }
            glutin::event::WindowEvent::CursorEntered { .. } => {
                self.events.push(InputEvent::Mouse(MouseEvent::Entered))
            }
            glutin::event::WindowEvent::CursorLeft { .. } => {
                self.events.push(InputEvent::Mouse(MouseEvent::Exited))
            }
            #[allow(deprecated)] // lol, uwu
            glutin::event::WindowEvent::MouseInput {
                state,
                button,
                modifiers,
                ..
            } => {
                let event = MouseEvent::Clicked(
                    map_mouse_button(*button),
                    map_elem_state(*state),
                    map_modifiers(*modifiers),
                );
                self.events.push(InputEvent::Mouse(event));
            }
            glutin::event::WindowEvent::MouseWheel { delta, .. } => match *delta {
                glutin::event::MouseScrollDelta::LineDelta(x, y) => self
                    .events
                    .push(InputEvent::Mouse(MouseEvent::Scrolled(x, y))),
                glutin::event::MouseScrollDelta::PixelDelta(physical_pos) => {
                    let (x, y) = physical_pos.into();
                    self.events
                        .push(InputEvent::Mouse(MouseEvent::Scrolled(x, y)));
                }
                _ => (),
            },
            glutin::event::WindowEvent::Resized(sz) => {
                self.events.push(InputEvent::Window(WindowEvent::Resized {
                    width: sz.width,
                    height: sz.height,
                }));
            }
            _ => (),
        }
    }
}

fn map_mouse_button(button: glutin::event::MouseButton) -> MouseButton {
    use glutin::event::MouseButton as Gm;
    match button {
        Gm::Left => MouseButton::Left,
        Gm::Middle => MouseButton::Middle,
        Gm::Right => MouseButton::Right,
        Gm::Other(u) => MouseButton::Other(u),
    }
}

fn map_modifiers(state: glutin::event::ModifiersState) -> ModifiersState {
    ModifiersState {
        shift: state.shift(),
        ctrl: state.ctrl(),
        alt: state.alt(),
        logo: state.logo(),
    }
}

fn map_elem_state(state: glutin::event::ElementState) -> ElementState {
    match state {
        glutin::event::ElementState::Pressed => ElementState::Pressed,
        glutin::event::ElementState::Released => ElementState::Released,
    }
}

fn map_keycode(key: glutin::event::VirtualKeyCode) -> KeyCode {
    use glutin::event::VirtualKeyCode as Gk;
    match key {
        Gk::Key1 => KeyCode::Key1,
        Gk::Key2 => KeyCode::Key2,
        Gk::Key3 => KeyCode::Key3,
        Gk::Key4 => KeyCode::Key4,
        Gk::Key5 => KeyCode::Key5,
        Gk::Key6 => KeyCode::Key6,
        Gk::Key7 => KeyCode::Key7,
        Gk::Key8 => KeyCode::Key8,
        Gk::Key9 => KeyCode::Key9,
        Gk::Key0 => KeyCode::Key0,

        Gk::A => KeyCode::A,
        Gk::B => KeyCode::B,
        Gk::C => KeyCode::C,
        Gk::D => KeyCode::D,
        Gk::E => KeyCode::E,
        Gk::F => KeyCode::F,
        Gk::G => KeyCode::G,
        Gk::H => KeyCode::H,
        Gk::I => KeyCode::I,
        Gk::J => KeyCode::J,
        Gk::K => KeyCode::K,
        Gk::L => KeyCode::L,
        Gk::M => KeyCode::M,
        Gk::N => KeyCode::N,
        Gk::O => KeyCode::O,
        Gk::P => KeyCode::P,
        Gk::Q => KeyCode::Q,
        Gk::R => KeyCode::R,
        Gk::S => KeyCode::S,
        Gk::T => KeyCode::T,
        Gk::U => KeyCode::U,
        Gk::V => KeyCode::V,
        Gk::W => KeyCode::W,
        Gk::X => KeyCode::X,
        Gk::Y => KeyCode::Y,
        Gk::Z => KeyCode::Z,

        Gk::Escape => KeyCode::Escape,

        Gk::F1 => KeyCode::F1,
        Gk::F2 => KeyCode::F2,
        Gk::F3 => KeyCode::F3,
        Gk::F4 => KeyCode::F4,
        Gk::F5 => KeyCode::F5,
        Gk::F6 => KeyCode::F6,
        Gk::F7 => KeyCode::F7,
        Gk::F8 => KeyCode::F8,
        Gk::F9 => KeyCode::F9,
        Gk::F10 => KeyCode::F10,
        Gk::F11 => KeyCode::F11,
        Gk::F12 => KeyCode::F12,
        Gk::F13 => KeyCode::F13,
        Gk::F14 => KeyCode::F14,
        Gk::F15 => KeyCode::F15,
        Gk::F16 => KeyCode::F16,
        Gk::F17 => KeyCode::F17,
        Gk::F18 => KeyCode::F18,
        Gk::F19 => KeyCode::F19,
        Gk::F20 => KeyCode::F20,
        Gk::F21 => KeyCode::F21,
        Gk::F22 => KeyCode::F22,
        Gk::F23 => KeyCode::F23,
        Gk::F24 => KeyCode::F24,

        Gk::Snapshot => KeyCode::Snapshot,
        Gk::Scroll => KeyCode::Scroll,
        Gk::Pause => KeyCode::Pause,

        Gk::Insert => KeyCode::Insert,
        Gk::Home => KeyCode::Home,
        Gk::Delete => KeyCode::Delete,
        Gk::End => KeyCode::End,
        Gk::PageDown => KeyCode::PageDown,
        Gk::PageUp => KeyCode::PageUp,

        Gk::Left => KeyCode::Left,
        Gk::Up => KeyCode::Up,
        Gk::Right => KeyCode::Right,
        Gk::Down => KeyCode::Down,

        Gk::Back => KeyCode::Back,
        Gk::Return => KeyCode::Return,
        Gk::Space => KeyCode::Space,

        Gk::Compose => KeyCode::Compose,

        Gk::Caret => KeyCode::Caret,

        Gk::Numlock => KeyCode::Numlock,
        Gk::Numpad0 => KeyCode::Numpad0,
        Gk::Numpad1 => KeyCode::Numpad1,
        Gk::Numpad2 => KeyCode::Numpad2,
        Gk::Numpad3 => KeyCode::Numpad3,
        Gk::Numpad4 => KeyCode::Numpad4,
        Gk::Numpad5 => KeyCode::Numpad5,
        Gk::Numpad6 => KeyCode::Numpad6,
        Gk::Numpad7 => KeyCode::Numpad7,
        Gk::Numpad8 => KeyCode::Numpad8,
        Gk::Numpad9 => KeyCode::Numpad9,
        Gk::NumpadAdd => KeyCode::NumpadAdd,
        Gk::NumpadDivide => KeyCode::NumpadDivide,
        Gk::NumpadDecimal => KeyCode::NumpadDecimal,
        Gk::NumpadComma => KeyCode::NumpadComma,
        Gk::NumpadEnter => KeyCode::NumpadEnter,
        Gk::NumpadEquals => KeyCode::NumpadEquals,
        Gk::NumpadMultiply => KeyCode::NumpadMultiply,
        Gk::NumpadSubtract => KeyCode::NumpadSubtract,

        Gk::AbntC1 => KeyCode::AbntC1,
        Gk::AbntC2 => KeyCode::AbntC2,
        Gk::Apostrophe => KeyCode::Apostrophe,
        Gk::Apps => KeyCode::Apps,
        Gk::Asterisk => KeyCode::Asterisk,
        Gk::At => KeyCode::At,
        Gk::Ax => KeyCode::Ax,
        Gk::Backslash => KeyCode::Backslash,
        Gk::Calculator => KeyCode::Calculator,
        Gk::Capital => KeyCode::Capital,
        Gk::Colon => KeyCode::Colon,
        Gk::Comma => KeyCode::Comma,
        Gk::Convert => KeyCode::Convert,
        Gk::Equals => KeyCode::Equals,
        Gk::Grave => KeyCode::Grave,
        Gk::Kana => KeyCode::Kana,
        Gk::Kanji => KeyCode::Kanji,
        Gk::LAlt => KeyCode::LAlt,
        Gk::LBracket => KeyCode::LBracket,
        Gk::LControl => KeyCode::LControl,
        Gk::LShift => KeyCode::LShift,
        Gk::LWin => KeyCode::LWin,
        Gk::Mail => KeyCode::Mail,
        Gk::MediaSelect => KeyCode::MediaSelect,
        Gk::MediaStop => KeyCode::MediaStop,
        Gk::Minus => KeyCode::Minus,
        Gk::Mute => KeyCode::Mute,
        Gk::MyComputer => KeyCode::MyComputer,
        Gk::NavigateForward => KeyCode::NavigateForward,
        Gk::NavigateBackward => KeyCode::NavigateBackward,
        Gk::NextTrack => KeyCode::NextTrack,
        Gk::NoConvert => KeyCode::NoConvert,
        Gk::OEM102 => KeyCode::OEM102,
        Gk::Period => KeyCode::Period,
        Gk::PlayPause => KeyCode::PlayPause,
        Gk::Plus => KeyCode::Plus,
        Gk::Power => KeyCode::Power,
        Gk::PrevTrack => KeyCode::PrevTrack,
        Gk::RAlt => KeyCode::RAlt,
        Gk::RBracket => KeyCode::RBracket,
        Gk::RControl => KeyCode::RControl,
        Gk::RShift => KeyCode::RShift,
        Gk::RWin => KeyCode::RWin,
        Gk::Semicolon => KeyCode::Semicolon,
        Gk::Slash => KeyCode::Slash,
        Gk::Sleep => KeyCode::Sleep,
        Gk::Stop => KeyCode::Stop,
        Gk::Sysrq => KeyCode::Sysrq,
        Gk::Tab => KeyCode::Tab,
        Gk::Underline => KeyCode::Underline,
        Gk::Unlabeled => KeyCode::Unlabeled,
        Gk::VolumeDown => KeyCode::VolumeDown,
        Gk::VolumeUp => KeyCode::VolumeUp,
        Gk::Wake => KeyCode::Wake,
        Gk::WebBack => KeyCode::WebBack,
        Gk::WebFavorites => KeyCode::WebFavorites,
        Gk::WebForward => KeyCode::WebForward,
        Gk::WebHome => KeyCode::WebHome,
        Gk::WebRefresh => KeyCode::WebRefresh,
        Gk::WebSearch => KeyCode::WebSearch,
        Gk::WebStop => KeyCode::WebStop,
        Gk::Yen => KeyCode::Yen,
        Gk::Copy => KeyCode::Copy,
        Gk::Paste => KeyCode::Paste,
        Gk::Cut => KeyCode::Cut,
    }
}
