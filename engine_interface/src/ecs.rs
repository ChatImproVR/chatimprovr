use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub struct Query;

pub struct QueryResult;

/// Universally-unique Entity ID
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityId(pub u128);

/// Component ID
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

/// Single command to be sent to engine
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub(crate) enum EngineCommand {
    Delete(EntityId),
}
