//! # ChatImproVR engine interface
//! This crate facilitates communication between **Plugins** and the **Host**.
//! It does not include any interfacing with the specific features of the **Client** or **Server**; instead these datatypes are relegated to the `common` crate.
//!
//! The entry point for **Plugins** is the [make_app_state!()](make_app_state) macro.

/// Code specific to WASM plugins
pub mod plugin;

pub use log;
/// Printing functions for plugins
pub mod stdout;

pub mod ecs;

/// Serialization format for plugin to host communication and vice versa
pub mod serial;

pub mod channels;

/// Systems and scheduling
pub mod system;

/// Networking
pub mod network;

/// PCG algorithm for generating random universally-unique entity IDs
pub mod pcg;

/// Convenience imports for the lazy
// #[macro_use]
pub mod prelude {
    pub use super::channels::*;
    pub use super::ecs::*;
    pub use super::log::*;
    pub use super::network::*;
    pub use super::plugin::*;
    pub use super::stdout::*;
    pub use super::system::*;
}
// TODO: Is this a million dollar mistake?
// It might be better to be explicit about it. Let people be lazy by making their own specialized
// macros.
/// Shorthand for `"<your crate's name>/$name"`, useful for namespaced IDs
#[macro_export]
macro_rules! pkg_namespace {
    ($name:expr) => {
        concat!(env!("CARGO_PKG_NAME"), "/", $name)
    };
}

/// Marks an ECS entity as persistent between plugin restarts
///
/// Server-side, this will be saved to disk too.
///
/// Client-side these are not saved to disk, but they are still useful for plugins maintaining
/// local ECS data in between plugin reloads
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Saved;

use ecs::{Component, ComponentIdStatic};
use prelude::{ChannelIdStatic, Locality, Message};
use serde::{Deserialize, Serialize};

impl Component for Saved {
    const ID: ComponentIdStatic = ComponentIdStatic {
        id: pkg_namespace!("Saved"),
        size: 0,
    };
}

/// Frame information
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct FrameTime {
    /// Delta time, in seconds
    pub delta: f32,
    /// Time since engine start, in seconds
    pub time: f32,
}

impl Message for FrameTime {
    const CHANNEL: ChannelIdStatic = ChannelIdStatic {
        id: pkg_namespace!("FrameTime"),
        locality: Locality::Local,
    };
}
