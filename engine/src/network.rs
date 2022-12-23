use cimvr_engine_interface::{prelude::MessageData, serial::EcsData};
use serde::{Deserialize, Serialize};

/// Message packet sent from server to client(s)
#[derive(Clone, Serialize, Deserialize)]
pub struct ServerToClient {
    /// All ECS data with an associated `Synchronized` component attached
    pub ecs: EcsData,
    pub messages: Vec<MessageData>,
}

/// Message packet sent from client to server
pub struct ClientToServer {
    pub messages: Vec<MessageData>,
}
