use std::collections::HashMap;

use crate::pkg_namespace;
use crate::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: Add metadata for edit requests, so that servers can intelligently filter...

/// A dynamic edit request sent to the server
#[derive(Message, Serialize, Deserialize, Clone)]
#[locality("Remote")]
pub struct DynamicEditRequest(pub DynamicEdit);

/// A dynamic edit command sent to the engine
#[derive(Message, Serialize, Deserialize, Clone)]
#[locality("Local")]
pub struct DynamicEditCommand(pub DynamicEdit);

/// A dynamic edit operation
#[derive(Serialize, Deserialize, Clone)]
pub struct DynamicEdit {
    /// Target entity
    pub entity: EntityId,
    /// Full component data state of this entity
    pub components: HashMap<ComponentId, Vec<u8>>,
}
