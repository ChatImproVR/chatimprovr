use cimvr_engine::{
    interface::{
        kobble::Schema,
        prelude::{Access, ComponentId, QueryComponent},
        ComponentSchema,
    },
    Engine,
};
use egui::{Context, ScrollArea};
use std::collections::{HashMap, HashSet};

pub struct ComponentUi {
    schema: HashMap<ComponentId, Schema>,
    selected: HashSet<ComponentId>,
}

impl ComponentUi {
    pub fn new(engine: &mut Engine) -> Self {
        engine.subscribe::<ComponentSchema>();
        Self {
            schema: Default::default(),
            selected: Default::default(),
        }
    }

    pub fn run(&mut self, ctx: &Context, engine: &mut Engine) {
        egui::SidePanel::left("ComponentUi").show(ctx, |ui| {
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
                }
            }
            ui.separator();

            //ScrollArea::vertical().show(ui, |ui| {
            /*
            for selection in &self.schema {
                let entities = engine.ecs().query(&[QueryComponent {
                    component: id.clone(),
                    access: Access::Write,
                }]);
                ui.label(&id.id);
                for ent in &entities {
                    ui.label(format!("    {:?}", ent));
                }
                //ui.label(&format!("{:?}", schema));
            }
            */
            //});
        });
    }

    pub fn update(&mut self, engine: &mut Engine) {
        for msg in engine.inbox::<ComponentSchema>() {
            let ComponentSchema { id, schema } = msg;
            self.schema.insert(id, schema);
        }
    }
}
