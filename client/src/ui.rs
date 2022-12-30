use std::collections::HashMap;

use cimvr_common::ui::*;
use cimvr_engine::Engine;
use egui::{Context, Ui};

pub struct OverlayUi {
    elements: HashMap<UiHandle, Element>,
}

struct Element {
    name: String,
    schema: Vec<Schema>,
    state: Vec<State>,
}

impl OverlayUi {
    pub fn new(engine: &mut Engine) -> Self {
        engine.subscribe::<UiRequest>();
        Self {
            elements: HashMap::new(),
        }
    }

    pub fn run(&mut self, ctx: &Context, engine: &mut Engine) {
        egui::SidePanel::left("my_side_panel").show(ctx, |ui| {
            for (id, elem) in self.elements.iter_mut() {
                if elem.show(ui) {
                    engine.send(UiUpdate {
                        id: *id,
                        state: elem.state.clone(),
                    });
                }
            }
        });
    }

    pub fn update(&mut self, engine: &mut Engine) {
        // Process requests
        for req in engine.inbox::<UiRequest>() {
            self.process_request(req);
        }
    }

    fn process_request(&mut self, req: UiRequest) {
        match req.op {
            UiOperation::Create {
                name,
                schema,
                init_state,
            } => {
                let elem = Element {
                    name,
                    schema,
                    state: init_state,
                };
                if self.elements.insert(req.id, elem).is_some() {
                    log::trace!("Replaced Ui element {:?}", req.id)
                }
            }
            UiOperation::Update(state) => {
                if let Some(elem) = self.elements.get_mut(&req.id) {
                    elem.state = state;
                } else {
                    log::error!("Failed to update invalid Ui element {:?}", req.id)
                }
            }
            UiOperation::Delete => {
                if self.elements.remove(&req.id).is_none() {
                    log::error!("Failed to remove invalid Ui element {:?}", req.id)
                }
            }
        }
    }
}

impl Element {
    /// Returns `true` if the given state updated
    pub fn show(&mut self, ui: &mut Ui) -> bool {
        todo!()
    }
}
