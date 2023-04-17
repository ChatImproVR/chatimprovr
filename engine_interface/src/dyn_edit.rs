use std::collections::HashMap;

use crate::pkg_namespace;
use crate::prelude::*;
use serde::{Deserialize, Serialize};

// TODO: Add metadata for edit requests, so that servers can intelligently filter...

/// A dynamic edit request sent to the server
/// Emitted by e.g. the component GUI to change entities containing the Synchronized component.
#[derive(Message, Serialize, Deserialize, Clone)]
#[locality("Remote")]
pub struct DynamicEditRequest(pub DynamicEdit);

/// A dynamic edit command sent to the engine
/// Emitted by e.g. the component GUI to locally change entities
#[derive(Message, Serialize, Deserialize, Clone)]
#[locality("Local")]
pub struct DynamicEditCommand(pub DynamicEdit);

/// A dynamic edit operation
#[derive(Serialize, Deserialize, Clone)]
pub struct DynamicEdit {
    /// Target entity
    pub entity: EntityId,
    // TODO: This is a potentially harmful thing to expose.
    // We should provide a wrapper around it!
    /// Full component data state of this entity
    pub components: HashMap<ComponentId, Vec<u8>>,
}
