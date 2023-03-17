use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

mod client;
mod server;
use client::ClientState;
use server::ServerState;

make_app_state!(ClientState, ServerState);

#[derive(Message, Clone, Debug, Serialize, Deserialize)]
#[locality("Remote")]
pub struct ChangeColor {
    rgb: [f32; 3],
}
