//! Types used for communication with the engine
use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// A description of a system within this plugin
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SystemDescriptor {
    /// The stage controls when this system is executed relative to engine functions
    pub stage: Stage,
    /// Channels this system subscribes to
    pub subscriptions: Vec<ChannelId>,
    /// Component queries
    pub query: Query,
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
    /// Keyboard and other Input is received in this stage
    Input,
    /// Physics data is processed in this stage
    Physics,
    /// Graphics and Sound are processsed in this stage
    Media,
}

impl Default for Stage {
    fn default() -> Self {
        Self::Input
    }
}

/*
/// Plugins may only be executed before
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Adjacency {
    Before,
    After,
}
*/