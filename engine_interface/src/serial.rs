//! Types used for communication with the engine
use std::io::{Write, Read};

use crate::plugin::EngineSchedule;
use crate::prelude::*;
use bincode::Options;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineIntrinsics {
    /// Random 64-bit number provided by the host
    pub random: u64,
}

/// Data transferred from Host to Plugin
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiveBuf {
    /// Which system to execute
    pub system: usize,
    /// Entity IDs aligned with components
    pub entities: Vec<EntityId>,
    /// Component data, with components in the same order as the query terms
    pub components: Vec<Vec<u8>>,
    /// Message buffers, in the same order as the subscribed channels
    pub messages: Vec<Vec<Message>>,
    /// Engine intrinsics
    pub intrinsics: EngineIntrinsics,
}

/// Data transferred from Plugin to Host
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendBuf {
    /// Commands to be sent to server
    pub commands: Vec<EngineCommand>,
    /// Messages to be sent to other plugins
    pub messages: Vec<Message>,
    /// Schedule setup on init. Must be empty except for first use!
    pub sched: Vec<SystemDescriptor>,
}

/// A description of a system within this plugin
#[derive(Clone, Debug, Serialize, Deserialize)]
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

pub fn serialize<W: Write, T: Serialize>(w: W, val: &T) -> bincode::Result<()> {
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
