use serde::{Deserialize, Serialize};

/// Code specific to WASM plugins
#[cfg(feature = "wasm-plugin")]
pub mod plugin;

/// Printing functions for plugins
pub mod stdout;

/// ECS interfacing types
pub mod ecs;

/// Serialization format for plugin to host communication and vice versa
pub mod serial;

/// Message channels
pub mod channels;

/// Systems and scheduling
pub mod system;

/// Networking
pub mod network;

/// PCG algorithm for generating random universally-unique entity IDs
pub(crate) mod pcg;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locality {
    /// Data are contained locally, and will never be transported over the network
    Local,
    /// Data are communicated between server and client
    Remote,
    //Remote(Reliability),
}

/*
pub enum Reliability {
    /// UDP-like
    Unreliable,
    /// TCP-like
    Reliable
}

impl Default for Reliability {
    fn default() -> Self {
        Reliability::Reliable
    }
}
*/

/// Convenience imports for the lazy
pub mod prelude {
    pub use super::channels::*;
    pub use super::ecs::*;
    pub use super::stdout::*;
    pub use super::system::*;
    pub use super::Locality;
    //pub use crate::{print, println};

    #[cfg(feature = "wasm-plugin")]
    pub use super::plugin::*;
}
