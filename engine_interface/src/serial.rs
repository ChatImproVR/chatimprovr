//! Types used for communication with the engine
use crate::plugin::EngineSchedule;
use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineIntrinsics {
    /// Random 64-bit number provided by the host
    pub random: u64,
}

/// Data transferred from Host to Plugin
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReceiveBuf {
    /// Entity IDs aligned with components
    pub entities: Vec<EntityId>,
    /// Component data, in the same order as the query
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
