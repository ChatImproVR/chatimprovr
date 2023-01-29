//! # Message channels
//! ChatImproVR follows the Pub/Sub messaging pattern.
//! After each **Stage**, the messages sent by the previous **Stage** are propagated to those
//! **Plugins** subscribed the to corresponding **Channel**.
//!
//! **Channels** come in two flavors, represented by [Locality](Locality).
//!

// //! The following example sends a message from a **Client** and receives it on the **Server**
// //!
// //! ```rust
// //!
// //! ```

use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::network::ClientId;

pub type Inbox = HashMap<ChannelId, Vec<MessageData>>;

/// Channel identity, corresponds to exactly one local or remote connection
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelId {
    /// Unique ID
    pub id: u128,
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
    pub client: Option<ClientId>,
    /// Message content
    pub data: Vec<u8>,
}

/// Trait denoting a Message type, which identifies a message by it's ID
///
/// See note about `CHANNEL`
pub trait Message: Serialize + DeserializeOwned + Sized {
    /// Channel ID
    ///
    /// Must be unique to the responsibility of this channel.
    /// You ***MUST*** change this ID if you change the datatype of this Message, to avoid
    /// sending corrupted data to other plugins
    const CHANNEL: ChannelId;
}
