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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locality {
    Local,
    Remote,
}

/// Convenience imports for the lazy
pub mod prelude {
    pub use super::channels::*;
    pub use super::ecs::*;
    pub use super::stdout::*;
    pub use crate::{printk, printlnk};

    #[cfg(feature = "wasm-plugin")]
    pub use super::plugin::*;
}
