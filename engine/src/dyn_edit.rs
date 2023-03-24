use crate::{ecs::Ecs, Engine};
use cimvr_engine_interface::{
    dyn_edit::{DynamicEdit, DynamicEditCommand},
    prelude::EntityId,
};

/// Extract a dynamic value from the ECS
pub fn extract_dyn(ecs: &Ecs, entity: EntityId) -> DynamicEdit {
    DynamicEdit {
        entity,
        components: ecs
            .all_components(entity)
            .map(|(c, d)| (c.clone(), d.to_vec()))
            .collect(),
    }
}

/// Insert a dynamic value into the ECS
pub fn insert_dyn(ecs: &mut Ecs, dynamic: &DynamicEdit) {
    // Delete existing state
    ecs.remove_entity(dynamic.entity);
    ecs.import_entity(dynamic.entity);

    // Import components
    for (comp_id, data) in &dynamic.components {
        ecs.add_component_raw(dynamic.entity, comp_id, data);
    }
}

/// Dynamic edit command follower
pub struct DynamicEditor;

impl DynamicEditor {
    pub fn new(engine: &mut Engine) -> Self {
        engine.subscribe::<DynamicEditCommand>();
        Self
    }

    /// Receive update events and apply them to the engine
    pub fn update(engine: &mut Engine) {
        for DynamicEditCommand(edit) in engine.inbox().collect::<Vec<_>>() {
            insert_dyn(engine.ecs(), &edit);
        }
    }
}
