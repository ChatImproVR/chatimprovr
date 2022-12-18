use cimvr_engine_interface::prelude::*;
use serde::{Deserialize, Serialize};
pub use winit::event::{ElementState, VirtualKeyCode};

/// Keyboard events
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyboardEvent {
    pub key: VirtualKeyCode,
    pub state: ElementState,
}

impl Message for KeyboardEvent {
    const CHANNEL: ChannelId = ChannelId {
        id: 0xC0DE_F00D,
        locality: Locality::Local,
    };
}
