use cimvr_engine::{
    interface::{
        kobble::Schema,
        prelude::{Access, ComponentId, EntityId, QueryComponent},
        ComponentSchema,
    },
    Engine,
};
use egui::{Context, ScrollArea};
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
            ScrollArea::vertical().show(ui, |ui| {
                for &entity in &self.display {
                    ui.label(format!("{:?}", entity));
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
