use cimvr_engine_interface::prelude::*;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

// TODO: FxHash

type ComponentData = Vec<u8>;
type EcsMap = HashMap<ComponentId, HashMap<EntityId, ComponentData>>;

/// Rather poor ECS implementation for prototyping
pub struct Ecs {
    map: EcsMap,
    entities: HashSet<EntityId>,
}

impl Ecs {
    /// Creates a new ECS world
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            entities: HashSet::new(),
        }
    }

    /// Returns the list of relevant entities
    /// Empty queries panic.
    /// If a component requested does not exist, panic.
    pub fn query(&self, query: &[QueryComponent]) -> HashSet<EntityId> {
        //let (init, rest) = q.split_first().expect("Empty query");
        let Some((init, rest)) = query.split_first() else { return HashSet::new() };

        // Initialize to the entities in the first term..
        // TODO: Pick smallest?
        let mut entities: HashSet<EntityId> = self
            .map
            .get(&init.component)
            .expect("Component does not exist")
            .keys()
            .copied()
            .collect();

        // Filter for the rest
        for term in rest {
            entities.retain(|ent| self.map[&term.component].contains_key(&ent));
        }

        entities
    }

    /// Import an entity ID from elsewhere
    pub fn import_entity(&mut self, id: EntityId) {
        self.entities.insert(id);
    }

    /// Remove an existing entity
    pub fn remove_entity(&mut self, id: EntityId) {
        for component in self.map.values_mut() {
            component.remove(&id);
        }

        let did_remove = self.entities.remove(&id);
        if !did_remove {
            eprintln!("Warning: Attempted to remove non-existant entity {:#?}", id);
        }
    }

    /// Create a new entity
    pub fn create_entity(&mut self) -> EntityId {
        // TODO: While new entity id is not in entities loop...
        let id = EntityId(rand::thread_rng().gen());
        self.entities.insert(id);
        id
    }

    /// Add component to entity, or overwrite existing data
    pub fn add_component(&mut self, entity: EntityId, component: ComponentId, data: &[u8]) {
        assert_eq!(data.len(), component.size as usize, "");
        assert!(
            self.entities.contains(&entity),
            "Entity not found, cannot add component"
        );

        // Get the component
        let comp = self.map.entry(component).or_default();

        // Set the data on the component
        if let Some(buf) = comp.get_mut(&entity) {
            buf.copy_from_slice(data);
        } else {
            comp.insert(entity, data.to_vec());
        }
    }

    /// Remove the given component from the given entity
    pub fn remove_component(&mut self, entity: EntityId, component: ComponentId) -> Vec<u8> {
        self.map
            .get_mut(&component)
            .expect("Missing entity")
            .remove(&entity)
            .expect("Entity missing component")
    }

    /// Get data associated with a component
    pub fn get(&self, entity: EntityId, component: ComponentId) -> &[u8] {
        self.map
            .get(&component)
            .expect("Missing entity")
            .get(&entity)
            .expect("Entity missing component")
    }

    /// Get data associated with a component
    pub fn get_mut(&mut self, entity: EntityId, component: ComponentId) -> &mut [u8] {
        self.map
            .get_mut(&component)
            .expect("Missing entity")
            .get_mut(&entity)
            .expect("Entity missing component")
    }

    /// Get all entities and data associated with the given component
    pub fn fast_all_component(
        &self,
        comp: ComponentId,
    ) -> impl Iterator<Item = (&EntityId, &ComponentData)> {
        self.map
            .get(&comp)
            .expect("Component does not exist")
            .iter()
    }

    /// Get all entities and data associated with the given component
    pub fn fast_all_component_mut(
        &mut self,
        comp: ComponentId,
    ) -> impl Iterator<Item = (&EntityId, &mut ComponentData)> {
        self.map
            .get_mut(&comp)
            .expect("Component does not exist")
            .iter_mut()
    }

    /*
    /// Get all data associated with a component
    pub get_all(&mut self, component: ComponentId) -> impl Iterator<Item=(EntityId, &ComponentData)> {
    }
    */

    /// Estimate bytes used by all component storage. Does not include ECS overhead.
    pub fn estimate_mem_usage(&self) -> usize {
        self.map
            .iter()
            .map(|(id, comp)| comp.len() * id.size as usize)
            .sum::<usize>()
    }

    /*
    /// Return a new world containing the queried entities and all appropriate components,
    /// removing them from our own world
    pub fn divide(&mut self, query: &[QueryTerm]) -> Self {
        let mut new_map: EcsMap = HashMap::new();

        let entities = self.query(query);

        // For each component
        for term in query {
            // Get matching component in source
            let comp = self.map.get_mut(&term.component).unwrap();

            // And move all relevant entities to the new map
            for &ent in &entities {
                new_map
                    .entry(term.component)
                    .or_default()
                    .insert(ent, comp.remove(&ent).unwrap());
            }
        }

        Self {
            map: new_map,
            entities,
        }
    }
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecs_basic() {
        let mut ecs = Ecs::new();

        let comp_a = ComponentId {
            id: 0xDEADBEEF,
            size: 8,
        };

        let test_val = 0x1337_3621_0420_6969_u64;
        let e = ecs.create_entity();
        ecs.add_component(e, comp_a, &test_val.to_le_bytes());

        let entities = ecs.query(&[QueryComponent {
            component: comp_a,
            access: Access::Read,
        }]);

        for ent in entities {
            let buf = ecs.get(ent, comp_a);
            let val = u64::from_le_bytes(buf.try_into().unwrap());
            assert_eq!(val, test_val);
            println!("{:X}", val);
        }
    }

    #[test]
    fn test_ecs_intermediate() {
        let mut ecs = Ecs::new();

        let comp_a = ComponentId {
            id: 0xDEADBEEF,
            size: 8,
        };

        let comp_b = ComponentId {
            id: 0xB00FCAFE,
            size: 8,
        };

        for i in 0..100u64 {
            let e = ecs.create_entity();
            if i < 50 {
                ecs.add_component(e, comp_b, &i.to_le_bytes());
            }
            ecs.add_component(e, comp_a, &0x1337_3621_0420_6969_u64.to_le_bytes());
        }

        let entities = ecs.query(&[
            QueryComponent {
                component: comp_a,
                access: Access::Read,
            },
            QueryComponent {
                component: comp_b,
                access: Access::Read,
            },
        ]);

        let mut showed_up = vec![false; 50];
        for ent in entities {
            let buf = ecs.get(ent, comp_b);
            let val = u64::from_le_bytes(buf.try_into().unwrap());
            showed_up[val as usize] = true;
        }

        dbg!(&showed_up);
        assert!(showed_up.iter().all(|&v| v), "But it was my birthday!!");

        let n_comp_a = ecs
            .query(&[QueryComponent {
                component: comp_a,
                access: Access::Read,
            }])
            .len();
        assert_eq!(n_comp_a, 100);

        let n_comp_b = ecs
            .query(&[QueryComponent {
                component: comp_b,
                access: Access::Read,
            }])
            .len();
        assert_eq!(n_comp_b, 50);
    }
}
