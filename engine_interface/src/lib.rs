//! # ChatImproVR engine interface
//! This crate facilitates communication between **Plugins** and the **Host**.
//! It does not include any interfacing with the specific features of the **Client** or **Server**; instead these datatypes are relegated to the `common` crate.
//!
//! The entry point for **Plugins** is the [make_app_state!()](make_app_state) macro.

/// Code specific to WASM plugins
pub mod plugin;

use std::{any::Any, cell::RefCell, collections::HashMap, marker::PhantomData};
mod component_validate_error;
pub mod component_validation;
use cimvr_derive_macros::Component;
pub use component_validation::is_fixed_size;

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
    pub use cimvr_derive_macros::{Component, Message};
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
#[derive(Component, Copy, Clone, Debug, Hash, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Saved;

/// Indicates saved data from this plugin
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SaveState<T>(Vec<u8>, PhantomData<T>);

use ecs::Component;
use once_cell::sync::Lazy;
use prelude::{ChannelIdStatic, ComponentId, Locality, Message};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serial::{deserialize, serialize, serialized_size};

// TODO: Use an integer of nanoseconds instead?
/// Frame timing information, denotes time since last frame
/// Note that a frame consists of PreUpdate, Update, and PostUpdate. This
/// time is captured before PreUpdate, and stays the same throughout.
#[derive(Message, Serialize, Deserialize, Debug, Clone, Copy)]
#[locality("Local")]
pub struct FrameTime {
    /// Delta time, in seconds
    pub delta: f32,
    /// Time since engine start, in seconds
    pub time: f32,
}

/// Get the maximum size of this component
#[track_caller]
fn max_component_size<C: Component>() -> usize {
    let component = C::default();
    validate_component(&component);
    serialized_size(&component).unwrap()
}

/// Validate that a component is fixed-size
#[track_caller]
fn validate_component<C: Component>(c: &C) {
    if let Err(err) = is_fixed_size(&c) {
        panic!(
            "The type {} is not fixed-size, and cannot be used as a component; {}",
            std::any::type_name::<C>(),
            err
        )
    }
}

/// Component size cache
pub(crate) struct SizeCache(HashMap<&'static str, usize>);

thread_local! {
    /// Thread local component size cache
    static SIZE_CACHE: RefCell<Lazy<SizeCache>> = RefCell::new(Lazy::new(|| SizeCache::new()));
}

impl SizeCache {
    pub fn new() -> Self {
        SizeCache(HashMap::new())
    }

    /// Get the size in bytse of the given component
    #[track_caller]
    pub fn size<C: Component>(&mut self) -> usize {
        let SizeCache(map) = self;
        *map.entry(C::ID)
            .or_insert_with(|| max_component_size::<C>())
    }
}

/// Get the size of a component
#[track_caller]
pub fn component_size_cached<C: Component>() -> usize {
    SIZE_CACHE.with(|cache| cache.borrow_mut().size::<C>())
}

/// Get the ComponentId of a Component
pub fn component_id<C: Component>() -> ComponentId {
    let size = component_size_cached::<C>()
        .try_into()
        .expect("Component is too large");
    ComponentId {
        id: C::ID.into(),
        size,
    }
}

impl<T: Serialize + DeserializeOwned> SaveState<T> {
    pub fn new(value: T) -> Self {
        Self(serialize(&value).unwrap(), PhantomData)
    }

    pub fn load(self) -> T {
        let SaveState(data, _) = self;
        deserialize(std::io::Cursor::new(&data)).unwrap()
    }
}

impl<T> Message for SaveState<T> {
    const CHANNEL: ChannelIdStatic = ChannelIdStatic {
        id: pkg_namespace!("SaveState"),
        locality: Locality::Local,
    };
}
