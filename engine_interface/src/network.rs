use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// Client connection identifier; unique to the connection and NOT the client.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client(pub u32);

/// Component indicating the entity is forcibly copied from client to server
/// Cannot be added to or removed from entities clientside!
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Synchronized;
//pub struct Synchronized(Reliability);

impl Component for Synchronized {
    const ID: ComponentId = ComponentId {
        id: 0x99999999999,
        size: 0,
    };
}

/*
TODO: Unreliable messaging
pub enum Reliability {
    /// UDP-like
    Unreliable,
    /// TCP-like
    Reliable
}

impl Default for Reliability {
    fn default() -> Self {
        Reliability::Reliable
    }
}
*/
