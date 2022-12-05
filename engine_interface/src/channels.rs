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
