//! # Entity Component System
//!
//!

use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    component_id,
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
#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub name: String,
    pub intersect: Vec<QueryComponent>,
}

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
    query: HashMap<String, Query>,
}

impl QueryResult {
    pub(crate) fn new(ecs: EcsData, query: HashMap<String, Query>) -> Self {
        Self {
            commands: vec![],
            ecs,
            query,
        }
    }

    /// Iterate through query entities
    pub fn iter(&self, name: &'static str) -> impl Iterator<Item = EntityId> {
        let query = &self.query[name];
        let (initial, xs) = query
            .intersect
            .split_first()
            .expect("No components in components");

        let tmp = HashMap::new();
        self.ecs
            .get(&initial.component)
            .unwrap_or_else(|| &tmp)
            .keys()
            .filter(|entity_id| {
                xs.iter().all(|q| match self.ecs.get(&q.component) {
                    Some(c) => c.contains_key(&entity_id),
                    None => false,
                })
            })
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
    }

    /*
    /// Get the index of the given entity id
    #[track_caller]
    fn get_entity_index(&self, entity: EntityId) -> usize {
        // TODO: This is slow!!!
        self.ecs
            .entities
            .iter()
            // .clone()
            // .into_iter()
            .position(|e| e.eq(&entity))
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
            .expect("Attempted to access component with invalid EntityID");

        let size = component_size_cached::<C>() as usize;
        let begin = entity_idx * size;
        let end = begin + size;

        (component_idx, begin..end)
    }
    */

    /// Read the data in the given component
    #[track_caller]
    pub fn read<C: Component>(&self, entity: EntityId) -> C {
        // TODO: Cache query lookups!
        let r = &self.ecs[&component_id::<C>()][&entity];
        deserialize(r.as_slice()).expect("Failed to deserialize component for reading")
    }

    /// Write the given data to the component
    #[track_caller]
    pub fn write<C: Component>(&mut self, entity: EntityId, component: &C) {
        let data = serialize(component).expect("Failed to serialize component for writing");

        // Write back to ECS storage for possible later modification. This is never read by the
        // host (for now), but MAY be read by the plugin!
        let w = self
            .ecs
            .get_mut(&component_id::<C>())
            .expect("Wrote to non-queried component id")
            .get_mut(&entity)
            .expect("Wrote to non-extant entity");
        *w = data.clone();

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

impl Query {
    /// Creates a new Query, assigning the label `name`.
    /// This name is used for both indexing, and for 
    pub fn new(name: &str) -> Self {
        Self::default().rename(name)
    }

    /// Renames this query. Useful in situations where you do not control the construction of the
    /// original query, e.g. from a library.
    pub fn rename(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Require this component to be present for each entity returned by this query.
    pub fn intersect<T: Component>(mut self, access: Access) -> Self {
        self.intersect.push(QueryComponent::new::<T>(access));
        self
    }
}
