//! # ChatImproVR engine interface
//! This crate facilitates communication between **Plugins** and the **Host**.
//! It does not include any interfacing with the specific features of the **Client** or **Server**; instead these datatypes are relegated to the `common` crate.
//!
//! The entry point for **Plugins** is the [make_app_state!()](make_app_state) macro.

/// Code specific to WASM plugins
pub mod plugin;

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
pub mod prelude {
    pub use super::channels::*;
    pub use super::ecs::*;
    pub use super::network::*;
    pub use super::plugin::*;
    pub use super::stdout::*;
    pub use super::system::*;
}

/// Shorthand for `"<your crate's name>/$name"`, useful for namespaced IDs
#[macro_export]
macro_rules! pkg_namespace {
    ($name:expr) => {
        concat!(env!("CARGO_PKG_NAME"), "/", $name)
    };
}
