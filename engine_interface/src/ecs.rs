//! # Entity Component System
//!
//!

use std::ops::Range;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    component_id, component_size_cached,
    serial::{deserialize, serialize, EcsData},
};

/// A single requirement in a query
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId {
    /// Universally-unique id
    pub id: String,
    /// Serialized size in bytes
    pub size: u16,
}

impl ComponentIdStatic {
    pub const fn new(id: &'static str, size: u16) -> Self {
        Self { id, size: 0 }
    }
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
pub trait Component: Serialize + DeserializeOwned + Copy + Default {
    /// Unique ID of this component
    const ID: &'static str;
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
            component: component_id::<T>(),
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

impl QueryResult {
    pub(crate) fn new(ecs: EcsData, query: Query) -> Self {
        Self {
            commands: vec![],
            ecs,
            query,
        }
    }

    /// Iterate through query entities
    pub fn iter(&self) -> impl Iterator<Item = EntityId> {
        self.ecs.entities.clone().into_iter()
    }

    /// Get the index of the given entity id
    #[track_caller]
    fn get_entity_index(&self, entity: EntityId) -> usize {
        // TODO: This is slow!!!
        self.ecs
            .entities
            .clone()
            .into_iter()
            .position(|e| e == entity)
            .expect("Attempted to access entity not queried")
    }

    /// Get the relevant component storage indices and
    #[track_caller]
    fn indices<C: Component>(&self, entity: EntityId) -> (usize, Range<usize>) {
        let entity_idx = self.get_entity_index(entity);

        let component_idx = self
            .query
            .iter()
            .position(|c| c.component.id == C::ID)
            .expect("Attempted to access component not queried");

        let size = component_size_cached::<C>() as usize;
        let begin = entity_idx * size;
        let end = begin + size;

        (component_idx, begin..end)
    }

    /// Read the data in the given component
    #[track_caller]
    pub fn read<T: Component>(&self, entity: EntityId) -> T {
        // TODO: Cache query lookups!
        let (component_idx, range) = self.indices::<T>(entity);
        let dense = &self.ecs.components[component_idx];
        deserialize(&dense[range]).expect("Failed to deserialize component for reading")
    }

    /// Write the given data to the component
    #[track_caller]
    pub fn write<C: Component>(&mut self, entity: EntityId, component: &C) {
        let entity_idx = self.get_entity_index(entity);
        let entity = self.ecs.entities[entity_idx];
        // Serialize data
        // TODO: Never allocate in hot loops!
        let data = serialize(component).expect("Failed to serialize component for writing");

        // Write back to ECS storage for possible later modification. This is never read by the
        // host, but MAY be read by us!
        let (component_idx, range) = self.indices::<C>(entity);
        let dense = &mut self.ecs.components[component_idx];
        dense[range].copy_from_slice(&data);

        // Write host command
        self.commands
            .push(EcsCommand::AddComponent(entity, component_id::<C>(), data))
    }

    // TODO: This is dreadfully slow but there's no way around that
    /// Modify the component "in place"
    #[track_caller]
    pub fn modify<T: Component>(&mut self, entity: EntityId, mut f: impl FnMut(&mut T)) {
        let mut val = self.read(entity);
        f(&mut val);
        self.write(entity, &mut val);
    }
}

/// Check that the given data size is compatible with this component
/// For now, the data size must be less than or equal to the prescribed size
#[track_caller]
pub fn check_component_data_size(component_size: u16, size: usize) {
    assert!(
        size <= usize::from(component_size),
        "Component data ({} bytes) must be less than or equal to the ID's size ({} bytes)",
        size,
        component_size
    );
}
