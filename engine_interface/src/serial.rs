use crate::plugin::EngineSchedule;
use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SerialSlice {
    /// Offset in bytes
    offset: usize,
    /// Length in bytes
    len: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MessageHeader {
    /// Channel ID
    channel: ChannelId,
    /// Return-address of message author
    author: AuthorId,
    /// Offset in buffer
    slice: SerialSlice,
}

/// Data received by plugin
#[derive(Clone, Debug, Serialize, Deserialize)]
struct PluginReceive {
    /// Location of ECS data. Location of actual data MUST NOT CHANGE!
    ecs: SerialSlice,
    /// Sanity check info on ECS data stride. Should match up!
    ecs_data_stride: usize,
    /// Indexing information for messages
    messages: Vec<SerialSlice>,
}

/// Data sent by plugin
#[derive(Clone, Debug, Serialize, Deserialize)]
struct PluginSend {
    /// Commands to be sent to server
    commands: Vec<EngineCommand>,
    /// Indexing information for messages
    messages: Vec<SerialSlice>,
}

/*
impl SerialData {
    pub fn write() -> Vec<u8> {
    }
}

fn iter_serial<U>(serial: &SerialData, sched: &EngineSchedule<U>, io: &mut EngineIo) {
}
*/
