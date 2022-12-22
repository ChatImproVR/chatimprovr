/// Code specific to WASM plugins
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

/// Convenience imports for the lazy
pub mod prelude {
    pub use super::channels::*;
    pub use super::ecs::*;
    pub use super::network::*;
    pub use super::plugin::*;
    pub use super::stdout::*;
    pub use super::system::*;
}
