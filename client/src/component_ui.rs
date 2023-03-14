use cimvr_engine::{Engine, interface::ComponentSchema};
use egui::Context;

pub struct ComponentUi {
}

impl ComponentUi {
    pub fn new(engine: &mut Engine) -> Self {
        engine.subscribe::<ComponentSchema>();
        Self {
        }
    }

    pub fn run(&mut self, ctx: &Context, engine: &mut Engine) {
    }

    pub fn update(&mut self, engine: &mut Engine) {
        for msg in engine.inbox::<ComponentSchema>() {
            dbg!(msg);
        }
    }
}
