//! # Message channels
//! ChatImproVR follows the Pub/Sub messaging pattern.
//! After each **Stage**, the messages sent by the previous **Stage** are propagated to those
//! **Plugins** subscribed the to corresponding **Channel**.
//!
//! **Channels** come in two flavors, represented by [Locality](Locality).
//!
//! See the `channels` example under `example_plugins` in the ChatImproVR repository

use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::network::ClientId;

pub type Inbox = HashMap<ChannelId, Vec<MessageData>>;

/// Channel identity, corresponds to exactly one local or remote connection
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelId {
    /// Unique ID
    pub id: String,
    /// Destination host; local or remote
    pub locality: Locality,
}

/// Channel identity, corresponds to exactly one local or remote connection
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ChannelIdStatic {
    /// Unique ID
    pub id: &'static str,
    /// Destination host; local or remote
    pub locality: Locality,
}

/// Determines whether messages are sent locally or to the remote
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locality {
    /// Messages are sent to or received from other plugins on the **Host**
    Local,
    /// Messages are sent to or received from the **Remote**
    Remote,
    //Remote(Reliability),
}

/// A single message sent or received
#[derive(Clone, Serialize, Deserialize)]
pub struct MessageData {
    /// Channel ID
    pub channel: ChannelId,
    /// Client ID
    /// * Will always be None for Locality::Local messages
    /// * Will always be None clientside
    /// * When received on server, contains ID of the client which sent it
    /// * When sent on a server, contains destination client ID if Some
    /// * Else broadcast to all if None
    pub client: Option<ClientId>,
    /// Message content
    pub data: Vec<u8>,
}

/// Trait denoting a Message type, which identifies a message by it's ID
///
/// # Arguments
///
/// * `CHANNEL` - Channel ID
///  
///
/// See note about `CHANNEL`
pub trait Message: Serialize + DeserializeOwned + Sized {
    /// Channel ID
    ///
    /// Must be unique to the responsibility of this channel.
    /// You ***MUST*** change this ID if you change the datatype of this Message, to avoid
    /// sending corrupted data to other plugins
    const CHANNEL: ChannelIdStatic;
}

impl From<ChannelIdStatic> for ChannelId {
    fn from(value: ChannelIdStatic) -> Self {
        Self {
            id: value.id.into(),
            locality: value.locality,
        }
    }
}

impl std::fmt::Debug for MessageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageData")
            .field("channel", &self.channel)
            .field("client", &self.client)
            .field("data (length)", &self.data.len())
            .finish()
    }
}
