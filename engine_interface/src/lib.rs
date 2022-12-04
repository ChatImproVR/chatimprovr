/// Code specific to WASM plugins
#[cfg(feature = "wasm-plugin")]
pub mod plugin;

/// ECS interfacing types
pub mod ecs;

/// Convenience imports for the lazy
pub mod prelude {
    #[cfg(feature = "wasm-plugin")]
    pub use super::plugin::*;

    pub use super::ecs::*;
}
