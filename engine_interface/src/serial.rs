//! Types used for communication with the engine
use std::io::{Read, Write};

use crate::prelude::*;
use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct EngineIntrinsics {
    /// Random 64-bit number provided by the host
    pub random: u64,
}

// (TODO: Make this just a header containing references to a dense buffer
/// Plugin-local ECS data
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
    /// Which system to execute
    pub system: usize,
    /// ECS data
    pub ecs: EcsData,
    /// Message buffers, in the same order as the subscribed channels
    pub messages: Vec<Vec<Message>>,
    /// Engine intrinsics
    pub intrinsics: EngineIntrinsics,
}

/// Data transferred from Plugin to Host
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SendBuf {
    /// Commands to be sent to server
    pub commands: Vec<EngineCommand>,
    /// Messages to be sent to other plugins
    pub messages: Vec<Message>,
    /// Schedule setup on init. Must be empty except for first use!
    pub sched: Vec<SystemDescriptor>,
    /// ECS data
    pub ecs: EcsData,
}

/// A description of a system within this plugin
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SystemDescriptor {
    /// Channels this plugin subscribes to
    pub subscriptions: Vec<ChannelId>,
    /// ECS query info
    pub query: Vec<QueryTerm>,
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

/*
impl SerialData {
    pub fn write() -> Vec<u8> {
    }
}

fn iter_serial<U>(serial: &SerialData, sched: &EngineSchedule<U>, io: &mut EngineIo) {
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SerialSlice {
    /// Offset in bytes
    offset: usize,
    /// Length in bytes
    len: usize,
}
*/
