use serde::{Deserialize, Serialize};

/// Client connection identifier; unique to the connection and NOT the client.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Client(pub u32);
