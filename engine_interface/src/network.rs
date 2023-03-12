use crate::{pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

/// Client connection identifier; unique to the connection and NOT the client.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientId(pub u32);

/// Component indicating the entity is forcibly copied from client to server
///
/// Cannot be added to or removed from entities clientside!
#[derive(Copy, Clone, Debug, Hash, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Synchronized;
//pub struct Synchronized(Reliability);

impl Component for Synchronized {
    const ID: &'static str = pkg_namespace!("Synchronized");
}

/// Information about a connected client
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    /// Unique connection identifier
    pub id: ClientId,
    /// Username supplied by the client
    pub username: String,
}

/// Message which lists currently connected clients. Available server-only
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connections {
    pub clients: Vec<Connection>,
}

impl Message for Connections {
    const CHANNEL: ChannelIdStatic = ChannelIdStatic {
        id: pkg_namespace!("Connections"),
        locality: Locality::Local,
    };
}

/// Connection request from client to server
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionRequest {
    pub version: u32,
    pub username: String,
}

impl ConnectionRequest {
    const PROTOCOL_VERSION: u32 = 1;

    /// Create a new connection request with the current protocol version
    pub fn new(username: String) -> Self {
        Self {
            version: Self::PROTOCOL_VERSION,
            username,
        }
    }

    /// Returns `true` if this connection request is valid for this version
    pub fn validate(&self) -> bool {
        self.version == Self::PROTOCOL_VERSION
    }
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
