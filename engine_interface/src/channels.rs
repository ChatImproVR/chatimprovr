use std::collections::HashMap;

use crate::Locality;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub type Inbox = HashMap<ChannelId, Vec<MessageData>>;

/// Channel identity, corresponds to exactly one local or remote connection
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelId {
    /// Unique ID
    pub id: u128,
    /// Locality
    pub locality: Locality,
}

/// A single message sent or received
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageData {
    /// Channel ID
    pub channel: ChannelId,
    /// Client ID
    /// * Will always be None for Locality::Local messages
    /// * Will always be None clientside
    /// * When received on server, contains ID of the client which sent it
    /// * When sent on a server, contains destination client ID if Some
    /// * Else broadcast to all if None
    pub client: Option<ChannelId>,
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
