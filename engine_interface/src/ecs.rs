use std::ops::Deref;

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

impl QueryResult {
    pub(crate) fn new(ecs: EcsData, query: Query) -> Self {
        Self {
            commands: vec![],
            ecs,
            query,
        }
    }

    fn iter_mut<'a>(&'a mut self) -> QueryIter<'a> {
        QueryIter::new(&mut self)
    }
}

pub struct QueryIter<'a> {
    data: &'a mut QueryResult,
    idx: usize,
}

impl QueryIter<'_> {
    fn new(data: &mut QueryResult) -> Self {
        Self { data, idx: 0 }
    }
}

impl<'a> Iterator for QueryIter<'a> {
    type Item = EntityRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let bound_check = self.idx > self.data.ecs.entities.len();
        bound_check.then(|| EntityRef {
            idx: self.idx,
            data: self.data,
        })
    }
}

pub struct EntityRef<'a> {
    idx: usize,
    data: &'a mut QueryResult,
}

impl EntityRef<'_> {
    pub fn entity(&self) -> EntityId {
        self.data.ecs.entities[self.idx]
    }

    pub fn read<T: Component>(&self) -> T {
        // TODO: Cache query lookups!
        let idx = self
            .data
            .query
            .iter()
            .position(|c| c.component == T::ID)
            .expect("Attempted to read component not queried");

        let dense = &mut self.data.ecs.components[idx];

        let size = T::ID.size as usize;
        let entry_slice = &dense[idx * size..][..size];

        deserialize(entry_slice).expect("Failed to deserialize component for reading")
    }

    pub fn write<T: Component>(&mut self, data: &T) {
        let entity = self.entity();
        let data = serialize(data).expect("Failed to serialize component for writing");
        self.data
            .commands
            .push(EngineCommand::AddComponent(entity, T::ID, data))
    }

    pub fn modify<T: Component>(&mut self, mut f: impl FnMut(&mut T)) {
        let mut val = self.read();
        f(&mut val);
        self.write(&mut val);
    }
}
