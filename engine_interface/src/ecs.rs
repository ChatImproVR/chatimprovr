use serde::{de::DeserializeOwned, Serialize};

/// Universally-unique Entity ID
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EntityId(u128);

/// Component ID
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ComponentId(u128);

/// Trait describing an ECS component
pub trait Component: Serialize + DeserializeOwned + Copy {
    const ID: ComponentId;
}
