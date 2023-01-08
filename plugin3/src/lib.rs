use cimvr_engine_interface::{make_app_state, prelude::*};
use serde::{Deserialize, Serialize};

mod client;
mod server;
use client::ClientState;
use server::ServerState;

make_app_state!(ClientState, ServerState);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeColor {
    rgb: [f32; 3],
}

impl Message for ChangeColor {
    const CHANNEL: ChannelId = ChannelId {
        id: 0x99999999999,
        locality: Locality::Remote,
    };
}
