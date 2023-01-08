use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::Result;
use cimvr_engine_interface::{
    prelude::*,
    serial::{deserialize, serialize, EcsData},
};
use rand::prelude::*;

// TODO: FxHash

pub type ComponentData = Vec<u8>;
pub type EcsMap = HashMap<ComponentId, HashMap<EntityId, ComponentData>>;

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
    pub fn query(&mut self, query: &[QueryComponent]) -> HashSet<EntityId> {
        //let (init, rest) = q.split_first().expect("Empty query");
        let Some((init, rest)) = query.split_first() else { return HashSet::new() };

        // Initialize to the entities in the first term..
        // TODO: Pick smallest?
        let mut entities: HashSet<EntityId> = self
            .map
            .entry(init.component)
            .or_default()
            .keys()
            .copied()
            .collect();

        // Filter for the rest
        for term in rest {
            entities.retain(|ent| match self.map.get(&term.component) {
                Some(comp_data) => comp_data.contains_key(&ent),
                None => false,
            });
        }

        entities
    }

    /// Returns a semi-random entity matching the given query, if any
    pub fn find(&mut self, query: &[ComponentId]) -> Option<EntityId> {
        // TODO: Optimize me
        let query: Vec<QueryComponent> = query
            .into_iter()
            .map(|c| QueryComponent {
                component: *c,
                access: Access::Read,
            })
            .collect();

        self.query(&query).drain().next()
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
            log::warn!("Attempted to remove non-existant entity {:#?}", id);
        }
    }

    /// Create a new entity
    pub fn create_entity(&mut self) -> EntityId {
        // TODO: While new entity id is not in entities loop...
        let id = EntityId(rand::thread_rng().gen());
        self.entities.insert(id);
        id
    }

    /// Convenient add component
    pub fn add_component<C: Component>(&mut self, entity: EntityId, data: &C) {
        self.add_component_raw(
            entity,
            C::ID,
            &serialize(data).expect("Failed to serialize component"),
        );
    }

    /// Add component to entity, or overwrite existing data
    pub fn add_component_raw(&mut self, entity: EntityId, component: ComponentId, data: &[u8]) {
        component.check_data_size(data.len());
        if !self.entities.contains(&entity) {
            return log::error!(
                "Failed to add component {:X?}; entity {:?} does not exist",
                component,
                entity
            );
        }

        // Get the component
        let comp = self.map.entry(component).or_default();

        // Set the data on the component
        // We ensure that the field is always the size of the component, and the remainder is
        // zeroed
        if let Some(buf) = comp.get_mut(&entity) {
            buf.fill(0);
            buf.copy_from_slice(data);
        } else {
            let mut v = data.to_vec();
            v.resize(usize::from(component.size), 0);
            comp.insert(entity, v);
        }
    }

    /// Remove the given component from the given entity
    pub fn remove_component(&mut self, entity: EntityId, component: ComponentId) {
        let Some(component) = self.map.get_mut(&component) else {
            return log::error!("Cannot remove from {:X?} {:X?} does not exist", entity, component);
        };

        let comp = component.remove(&entity);

        if comp.is_none() {
            log::error!(
                "Entity {:X?} does not have component {:X?}",
                entity,
                component
            );
        }
    }

    /// Convenient get function
    pub fn get<C: Component>(&self, entity: EntityId) -> Option<C> {
        Some(deserialize(self.get_raw(entity, C::ID)?).expect("Failed to deserialize component"))
    }

    /// Get data associated with a component
    pub fn get_raw(&self, entity: EntityId, component: ComponentId) -> Option<&[u8]> {
        let Some(component) = self.map.get(&component) else {
            log::error!("Cannot get {:X?} (does not exist)", component);
            return None;
        };

        let comp = component.get(&entity);

        if comp.is_none() {
            log::error!(
                "Entity {:X?} does not have component {:X?}",
                entity,
                component
            );
        }

        comp.map(|s| s.as_slice())
    }

    /// Get data associated with a component
    pub fn get_mut(&mut self, entity: EntityId, component: ComponentId) -> Option<&mut [u8]> {
        let Some(column) = self.map.get_mut(&component) else {
            log::error!("Component {:?} does not exist", component);
            return None;
        };

        let comp = column.get_mut(&entity);

        if comp.is_none() {
            log::error!(
                "Entity {:?} does not have component {:?}",
                entity,
                component
            );
        }

        comp.map(|s| s.as_mut_slice())
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

    pub fn export(&mut self, query: &[QueryComponent]) -> EcsMap {
        let entities: Vec<EntityId> = self.query(query).into_iter().collect();

        let mut exp = EcsMap::new();
        for (id, comp) in &mut self.map {
            let map = exp.entry(*id).or_default();
            for ent in &entities {
                if let Some(data) = comp.get(ent) {
                    map.insert(*ent, data.clone());
                }
            }
        }

        exp
    }

    pub fn import(&mut self, query: &[QueryComponent], imported: EcsMap) {
        // Remove existing entities in the given query
        let entities: Vec<EntityId> = self.query(query).into_iter().collect();
        for id in entities {
            self.remove_entity(id);
        }

        // Add component data from import
        for (id, import_comp) in imported {
            let my_comp = self.map.entry(id).or_default();
            for (ent, data) in import_comp {
                my_comp.insert(ent, data);
                self.entities.insert(ent);
            }
        }
    }
}

/// Query the given ECS and serialize into ECSData
pub fn query_ecs_data(ecs: &mut Ecs, query: &Query) -> Result<EcsData> {
    let entities = ecs.query(query).into_iter().collect();
    let mut components = vec![vec![]; query.len()];

    for &entity in &entities {
        for (term, comp) in query.iter().zip(&mut components) {
            if let Some(v) = ecs.get_raw(entity, term.component) {
                comp.extend_from_slice(v);
            }
        }
    }

    Ok(EcsData {
        entities,
        components,
    })
}

/// Apply the given commands to the given ecs
pub fn apply_ecs_commands(ecs: &mut Ecs, commands: &[EcsCommand]) -> Result<()> {
    // Apply commands
    for command in commands {
        // TODO: Throw error on modification of non-queried data...
        match command {
            EcsCommand::Create(id) => ecs.import_entity(*id),
            EcsCommand::Delete(id) => ecs.remove_entity(*id),
            EcsCommand::AddComponent(entity, component, data) => {
                ecs.add_component_raw(*entity, *component, data)
            }
        }
    }

    Ok(())
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
        ecs.add_component_raw(e, comp_a, &test_val.to_le_bytes());

        let entities = ecs.query(&[QueryComponent {
            component: comp_a,
            access: Access::Read,
        }]);

        for ent in entities {
            let buf = ecs.get_raw(ent, comp_a);
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
                ecs.add_component_raw(e, comp_b, &i.to_le_bytes());
            }
            ecs.add_component_raw(e, comp_a, &0x1337_3621_0420_6969_u64.to_le_bytes());
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
            let buf = ecs.get_raw(ent, comp_b);
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
