use serde::{Deserialize, Serialize};

/// Code specific to WASM plugins
#[cfg(feature = "wasm-plugin")]
pub mod plugin;

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
    #[cfg(feature = "wasm-plugin")]
    pub use super::plugin::*;

    pub use super::channels::*;
    pub use super::ecs::*;
}
