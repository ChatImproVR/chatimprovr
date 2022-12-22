//! Types used for communication with the engine
use std::io::{Read, Write};

use crate::prelude::*;
use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// (TODO: Make this just a header containing references to a dense buffer
/// Plugin-local ECS data
/// Represents the data returned by a query
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EcsData {
    /// Entity IDs aligned with components
    pub entities: Vec<EntityId>,
    /// Component data SoA, with components in the same order as the query terms
    pub components: Vec<Vec<u8>>,
}

/// Data transferred from Host to Plugin
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ReceiveBuf {
    /// Which system to execute, None if initializing
    pub system: Option<usize>,
    /// Compact ECS data
    pub ecs: EcsData,
    /// Message inbox
    pub inbox: Inbox,
    /// True if plugin is server-side
    pub is_server: bool,
    /*
    /// Message buffers, in the same order as the subscribed channels
    pub messages: Vec<Vec<Message>>,
    */
}

/// Data transferred from Plugin to Host
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SendBuf {
    /// Commands to be sent to server
    pub commands: Vec<EngineCommand>,
    /*
    /// Messages to be sent to other plugins
    pub messages: Vec<Message>,
    */
    /// Schedule setup on init. Must be empty except for first use!
    pub systems: Vec<SystemDescriptor>,
    /// Message outbox
    pub outbox: Vec<MessageData>,
}

fn bincode_opts() -> impl Options {
    // NOTE: This is actually different from the default bincode serialize() function!!
    bincode::DefaultOptions::new()
}

pub fn serialize<T: Serialize>(val: &T) -> bincode::Result<Vec<u8>> {
    bincode_opts().serialize(val)
}

pub fn serialize_into<W: Write, T: Serialize>(w: W, val: &T) -> bincode::Result<()> {
    bincode_opts().serialize_into(w, val)
}

pub fn serialized_size<T: Serialize>(val: &T) -> bincode::Result<usize> {
    Ok(bincode_opts().serialized_size(val)? as usize)
}

pub fn deserialize<R: Read, T: DeserializeOwned>(r: R) -> bincode::Result<T> {
    bincode_opts().deserialize_from(r)
}
