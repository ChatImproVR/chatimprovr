use std::{fmt::Display, num::ParseIntError, str::FromStr};

use crate::{pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

/// Client connection identifier; unique to the connection and NOT the client.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientId(pub u32);

/// Component indicating the entity is forcibly copied from client to server
///
/// Cannot be added to or removed from entities clientside!
#[derive(Component, Copy, Clone, Debug, Hash, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Synchronized;
//pub struct Synchronized(Reliability);

/// Information about a connected client
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    /// Unique connection identifier
    pub id: ClientId,
    /// Username supplied by the client
    pub username: String,
}

/// Message which lists currently connected clients. Available server-only
#[derive(Message, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[locality("Local")]
pub struct Connections {
    pub clients: Vec<Connection>,
}

/// Connection request from client to server
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionRequest {
    pub version: u32,
    pub username: String,
    pub plugin_manifest: Vec<Digest>,
}

// TODO: Should this be part of `common`?
/// Connection request from client to server
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionResponse {
    /// Contains pairs of (name, code), corresponding to the plugins the server wants
    pub plugins: Vec<(String, PluginData)>,
}

/// Connection data
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginData {
    Cached(Digest),
    // TODO: Send compressed data!
    Download(Vec<u8>),
}

impl ConnectionRequest {
    const PROTOCOL_VERSION: u32 = 2;

    /// Create a new connection request with the current protocol version
    pub fn new(username: String, plugin_manifest: Vec<Digest>) -> Self {
        Self {
            version: Self::PROTOCOL_VERSION,
            plugin_manifest,
            username,
        }
    }

    /// Returns `true` if this connection request is valid for this version
    pub fn validate(&self) -> bool {
        self.version == Self::PROTOCOL_VERSION
    }
}

/// Hash of a file (xxHash3)
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Digest(pub u128);

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

impl Default for ClientId {
    fn default() -> Self {
        ClientId(0xBADBADBA)
    }
}

impl Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(hash) = self;
        write!(f, "{}", hash)
    }
}

impl FromStr for Digest {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}
