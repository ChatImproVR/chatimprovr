use std::ops::Deref;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

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
pub struct QueryTransaction {
    query: Query,
}

impl QueryTransaction {
    pub fn iter_mut(&mut self) -> QueryTransactionIterator<'static> {
        todo!()
    }
}

pub struct QueryTransactionIterator<'a> {
    trans: &'a mut QueryTransaction,
    query: Query,
}

impl<'a> Iterator for QueryTransactionIterator<'a> {
    type Item = QueryRow<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

pub struct QueryRow<'a> {
    trans: &'a mut QueryTransaction,
    query: Query,
}

impl QueryRow<'_> {
    pub fn entity(&self) -> EntityId {
        todo!()
    }

    pub fn read<T: Component>(&self) -> T {
        todo!()
    }

    pub fn write<T: Component>(&mut self, data: &T) {
        todo!()
    }

    pub fn modify<T: Component>(&mut self, mut f: impl FnMut(&mut T)) {
        let mut val = self.read();
        f(&mut val);
        self.write(&mut val);
    }
}
