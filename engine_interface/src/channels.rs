use crate::Locality;
use serde::{Deserialize, Serialize};

/// Channel identity, corresponds to exactly one local or remote connection
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelId {
    pub id: u128,
    pub locality: Locality,
}

/// Anonymous identity of peer plugin, for return messages
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorId(u32);

/// A single message sent or received
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    /// Channel ID
    pub channel: ChannelId,
    /// Return-address of message author. Always `Some()` when received,
    /// `None` will send to all potential recipients,
    /// `Some(id)` will send to just the given author. Useful for return messages
    pub author: Option<AuthorId>,
    /// Message content
    pub data: Vec<u8>,
}
