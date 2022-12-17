use std::collections::HashMap;

use crate::Locality;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub(crate) type Inbox = HashMap<ChannelId, Vec<MessageData>>;

/// Channel identity, corresponds to exactly one local or remote connection
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelId {
    pub id: u128,
    pub locality: Locality,
}

/*
/// Anonymous identity of peer plugin, for return messages
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorId(u32);
*/

/// A single message sent or received
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageData {
    /// Channel ID
    pub channel: ChannelId,
    /* TODO: Return addresses
    /// Return-address of message author. Always `Some()` when received,
    /// `None` will send to all potential recipients,
    /// `Some(id)` will send to just the given author. Useful for return messages
    pub author: Option<AuthorId>,
    */
    /// Message content
    pub data: Vec<u8>,
}

/// A single message sent or received
pub trait Message: Serialize + DeserializeOwned + Sized {
    /// Channel ID
    const CHANNEL: ChannelId;
}

/// Subscribe to the given channel
pub fn sub<M: Message>() -> ChannelId {
    M::CHANNEL
}
