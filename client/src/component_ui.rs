use std::collections::HashMap;
use cimvr_engine::{Engine, interface::{ComponentSchema, prelude::ComponentId, kobble::Schema}};
use egui::{Context, ScrollArea};

pub struct ComponentUi {
    schema: HashMap<ComponentId, Schema>,
}

impl ComponentUi {
    pub fn new(engine: &mut Engine) -> Self {
        engine.subscribe::<ComponentSchema>();
        Self {
            schema: Default::default(),
        }
    }

    pub fn run(&mut self, ctx: &Context, engine: &mut Engine) {
        egui::SidePanel::right("ComponentUi").show(ctx, |ui| {
            //ScrollArea::vertical().show(ui, |ui| {
                for (id, elem) in &self.schema {
                    ui.label(&id.id);
                }
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
