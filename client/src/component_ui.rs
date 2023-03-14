use cimvr_engine::{
    interface::{
        kobble::{Schema, SchemaDeserializer, DynamicValue},
        prelude::{Access, ComponentId, EntityId, QueryComponent},
        ComponentSchema, serial::{deserialize, serialize, serialize_into},
    },
    Engine,
};
use egui::{Context, ScrollArea, Ui};
use std::collections::{HashMap, HashSet};

pub struct ComponentUi {
    schema: HashMap<ComponentId, Schema>,
    selected: HashSet<ComponentId>,
    display: Vec<EntityId>,
}

impl ComponentUi {
    pub fn new(engine: &mut Engine) -> Self {
        engine.subscribe::<ComponentSchema>();
        Self {
            schema: Default::default(),
            selected: Default::default(),
            display: Default::default(),
        }
    }

    pub fn run(&mut self, ctx: &Context, engine: &mut Engine) {
        egui::SidePanel::left("ComponentUi").show(ctx, |ui| {
            // Component selector
            let mut needs_update = false;
            ui.label("Components:");
            for id in self.schema.keys() {
                let has_id = self.selected.contains(id);
                let marker = if has_id { '-' } else { '+' };
                let button = ui.button(format!("{}{}", marker, id.id));

                if button.clicked() {
                    if has_id {
                        self.selected.remove(id);
                    } else {
                        self.selected.insert(id.clone());
                    }
                    needs_update = true;
                }
            }
            ui.separator();

            // Update displayed entities
            // TODO: Actually update each frame? Just sort the ids.
            // Might get a bit jittery with lots of plugins adding/removing entities...
            if needs_update {
                let query: Vec<QueryComponent> = self
                    .selected
                    .iter()
                    .map(|id| QueryComponent {
                        component: id.clone(),
                        access: Access::Write,
                    })
                    .collect();

                self.display = engine.ecs().query(&query).into_iter().collect();
            }

            // Component editor
            let mut sorted_components: Vec<ComponentId> = self.selected.iter().cloned().collect();
            sorted_components.sort_by(|a, b| a.id.cmp(&b.id));

            ScrollArea::vertical().show(ui, |ui| {
                for &entity in &self.display {
                    ui.label(format!("{:?}", entity));
                    for component in &sorted_components {
                        let schema = self.schema[component].clone();
                        let data = engine.ecs().get_raw(entity, component).unwrap();

                        SchemaDeserializer::set_schema(schema);
                        if let Ok(SchemaDeserializer(mut dynamic)) = deserialize(std::io::Cursor::new(data)) {

                            ui.label(format!("{}", component.id));

                            if editor(&mut dynamic, ui) {
                                let data = engine.ecs().get_mut(entity, component).unwrap();
                                serialize_into(std::io::Cursor::new(data), &dynamic).unwrap();
                            }

                        } else {
                            ui.label(format!("Failed to deserialize {}", component.id));
                        }

                    }
                    ui.separator();
                }
            })
        });
    }

    pub fn update(&mut self, engine: &mut Engine) {
        for msg in engine.inbox::<ComponentSchema>() {
            let ComponentSchema { id, schema } = msg;
            self.schema.insert(id, schema);
        }
    }
}

fn editor(value: &mut DynamicValue, ui: &Ui) -> bool {
    false
}
