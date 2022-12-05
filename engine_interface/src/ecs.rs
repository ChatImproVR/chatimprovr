use serde::{de::DeserializeOwned, Serialize};

pub struct Query;

pub struct QueryResult;

/// Universally-unique Entity ID
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EntityId(pub u128);

/// Component ID
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ComponentId {
    /// Universally-unique id
    pub id: u128,
    /// Size in bytes
    pub size: u16,
}

/// Trait describing an ECS component
pub trait Component: Serialize + DeserializeOwned + Copy {
    const ID: ComponentId;
}
