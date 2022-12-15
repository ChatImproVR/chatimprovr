use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::serial::{deserialize, serialize, EcsData};

/// A single requirement in a query
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryTerm {
    /// Component ID queried
    pub component: ComponentId,
    /// Access level granted to this component
    pub access: Access,
}

/// A description of an ECS query
pub type Query = Vec<QueryTerm>;

/// Universally-unique Entity ID
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u128);

/// Component ID
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId {
    /// Universally-unique id
    pub id: u128,
    /// Size in bytes
    pub size: u16,
}

/// Access level for the given component
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Access {
    /// Read only. Writes will be discarded!
    Read,
    /// Read and write access
    Write,
}

/// Trait describing an ECS component
// Copy bound here is to discourage variable-sized types!
pub trait Component: Serialize + DeserializeOwned + Copy {
    /// Unique ID of this component
    const ID: ComponentId;
}

/// Single command to be sent to engine
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineCommand {
    Create(EntityId),
    Delete(EntityId),
    // For now we're betting that the user doesn't add that many entities at once...
    AddComponent(EntityId, ComponentId, Vec<u8>),
}

impl QueryTerm {
    pub fn new<T: Component>(access: Access) -> Self {
        Self {
            component: T::ID,
            access,
        }
    }
}

/// Read and write ECS data relevant to a query
pub struct QueryResult {
    /// For modifications. TODO: Use a faster method of update xfer...
    commands: Vec<EngineCommand>,
    /// ECS data from host
    ecs: EcsData,
    /// The original query, for reference
    query: Query,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Key {
    idx: usize,
    entity: EntityId,
}

impl Key {
    pub fn entity(&self) -> EntityId {
        self.entity
    }
}

impl QueryResult {
    pub(crate) fn new(ecs: EcsData, query: Query) -> Self {
        Self {
            commands: vec![],
            ecs,
            query,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Key> {
        self.ecs
            .entities
            .clone()
            .into_iter()
            .enumerate()
            .map(|(idx, entity)| Key { idx, entity })
    }

    pub fn read<T: Component>(&self, key: Key) -> T {
        // TODO: Cache query lookups!
        let component_idx = self
            .query
            .iter()
            .position(|c| c.component == T::ID)
            .expect("Attempted to read component not queried");

        let dense = &self.ecs.components[component_idx];

        let size = T::ID.size as usize;
        let entry_slice = &dense[key.idx * size..][..size];

        deserialize(entry_slice).expect("Failed to deserialize component for reading")
    }

    pub fn write<T: Component>(&mut self, key: Key, data: &T) {
        let entity = self.ecs.entities[key.idx];
        let data = serialize(data).expect("Failed to serialize component for writing");

        // TODO: Writeback to ECS data? May not ever be needed!

        self.commands
            .push(EngineCommand::AddComponent(entity, T::ID, data))
    }

    pub fn modify<T: Component>(&mut self, key: Key, mut f: impl FnMut(&mut T)) {
        let mut val = self.read(key);
        f(&mut val);
        self.write(key, &mut val);
    }
}
