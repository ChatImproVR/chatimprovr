//! Types used for communication with the engine
use std::collections::HashMap;

use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// A description of a system within this plugin
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemDescriptor {
    /// The stage controls when this system is executed relative to engine functions
    pub stage: Stage,
    /// Channels this system subscribes to
    pub subscriptions: Vec<ChannelId>,
    /// Component queries
    pub queries: HashMap<String, Query>,
}

/// This flag indicates which stage the plugin is to be executed **after**.
/// For example, the Input stage is executed after user input
/// The execution cycle of the engine is something like this:
/// * Sync ECS with server
/// * Collect keyboard, VR tracking input (`Stage::Input`)
/// * Physics (`Stage::Physics`)
/// * Graphics and sound (`Stage::Media`)
/// * Send messages to server
#[derive(Clone, Copy, Debug, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub enum Stage {
    /// Plugins are initialized before this stage. Runs every time *any* plugin is initialized
    PostInit,
    /// Keyboard and other Input is received before this stage
    PreUpdate,
    /// Physics data is processed before this stage
    Update,
    /// Graphics and Sound are processsed before this stage
    PostUpdate,
    /// Run when the plugin is about to be shutdown or reloaded and must save any state to memory
    Shutdown,
}

impl Default for SystemDescriptor {
    fn default() -> Self {
        Self {
            stage: Stage::Update,
            subscriptions: vec![],
            queries: Default::default(),
        }
    }
}
