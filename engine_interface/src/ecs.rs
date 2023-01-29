//! # Entity Component System
//!
//!

use std::ops::Range;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::serial::{deserialize, serialize, EcsData};

/// A single requirement in a query
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryComponent {
    /// Component ID queried
    pub component: ComponentId,
    /// Access level granted to this component
    pub access: Access,
}

/// A description of an ECS query
pub type Query = Vec<QueryComponent>;

/// Universally-unique Entity ID
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u128);

/// Component ID
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId {
    /// Universally-unique id
    pub id: u128,
    /// Serialized size in bytes
    ///
    /// Preemptive FAQ:
    /// Q: What? How do I get this number?
    /// A: Just put any ol number in and use it in `add_component()` once.
    /// The resulting crash will inform you of the size
    /// A: Or just use `engine_interface::serial::serialized_size()` at runtime
    ///
    /// Q: Why do you need this?
    /// A: So the engine can check that it's right
    /// A: To easily move components around in memory
    /// A: To help uniquely identify the database key with it's data type
    ///
    /// Q: Is this the same as the size of the associated type?
    /// A: Not always! `std::mem::size_of<T>()` (`where T: Component`)
    /// can be different than `serialized_size::<T>()`.
    /// Layout in memory subject to change
    ///
    /// Q: Why is this u16?
    /// A: Are you kidding? Components >64k? Are you outta your mind?!
    /// A: Honestly this should be u8 but I was merciful.
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
pub enum EcsCommand {
    Create(EntityId),
    Delete(EntityId),
    // TODO: For now we're betting that the user doesn't add that many components at once...
    AddComponent(EntityId, ComponentId, Vec<u8>),
}

impl QueryComponent {
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
    pub(crate) commands: Vec<EcsCommand>,
    /// ECS data from host
    ecs: EcsData,
    /// The original query, for reference
    query: Query,
}

// TODO: Use only EntityId instead of this method...
// We should be able to read/write specific ids!
/// Opaque key to access query indices
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

    /// Iterate through query entities
    pub fn iter(&self) -> impl Iterator<Item = Key> {
        self.ecs
            .entities
            .clone()
            .into_iter()
            .enumerate()
            .map(|(idx, entity)| Key { idx, entity })
    }

    /// Get the relevant component storage indices and
    #[track_caller]
    fn indices<T: Component>(&self, key: Key) -> (usize, Range<usize>) {
        let component_idx = self
            .query
            .iter()
            .position(|c| c.component == T::ID)
            .expect("Attempted to access component not queried");

        let size = T::ID.size as usize;
        let begin = key.idx * size;
        let end = begin + size;

        (component_idx, begin..end)
    }

    /// Read the data in the given component
    pub fn read<T: Component>(&self, key: Key) -> T {
        // TODO: Cache query lookups!
        let (component_idx, range) = self.indices::<T>(key);
        let dense = &self.ecs.components[component_idx];
        deserialize(&dense[range]).expect("Failed to deserialize component for reading")
    }

    /// Write the given data to the component
    pub fn write<C: Component>(&mut self, key: Key, data: &C) {
        let entity = self.ecs.entities[key.idx];
        // Serialize data
        // TODO: Never allocate in hot loops!
        let data = serialize(data).expect("Failed to serialize component for writing");
        C::ID.check_data_size(data.len());

        // Write back to ECS storage for possible later modification. This is never read by the
        // host, but MAY be read by us!
        let (component_idx, range) = self.indices::<C>(key);
        let dense = &mut self.ecs.components[component_idx];
        dense[range].copy_from_slice(&data);

        // Write host command
        self.commands
            .push(EcsCommand::AddComponent(entity, C::ID, data))
    }

    // TODO: This is dreadfully slow but there's no way around that
    /// Modify the component "in place"
    pub fn modify<T: Component>(&mut self, key: Key, mut f: impl FnMut(&mut T)) {
        let mut val = self.read(key);
        f(&mut val);
        self.write(key, &mut val);
    }
}

impl ComponentId {
    /// Check that the given data size is compatible with this component
    /// For now, the data size must be less than or equal to the prescribed size
    pub fn check_data_size(&self, size: usize) {
        assert!(
            size <= usize::from(self.size),
            "Component data ({} bytes) must be less than or equal to the ID's size ({} bytes)",
            size,
            self.size,
        );
    }
}
