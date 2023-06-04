use std::collections::HashMap;

use cimvr_common::ui::*;
use cimvr_engine::Engine;
use egui::{color_picker::color_edit_button_rgb, Context, DragValue, ScrollArea, TextEdit, Ui};

use crate::{component_ui::ComponentUi, plugin_ui::PluginUi};

pub struct OverlayUi {
    plugin_ui: PluginUi,
    component_ui: ComponentUi,
}

impl OverlayUi {
    pub fn new(engine: &mut Engine) -> Self {
        Self {
            plugin_ui: PluginUi::new(engine),
            component_ui: ComponentUi::new(engine),
        }
    }

    pub fn run(&mut self, ctx: &Context, engine: &mut Engine) {
        self.plugin_ui.run(ctx, engine);
        self.component_ui.run(ctx, engine);
    }

    pub fn update(&mut self, engine: &mut Engine) {
        self.plugin_ui.update(engine);
        self.component_ui.update(engine);
    }
}

/*
=======
        // Process requests
        for req in engine.inbox::<UiRequest>() {
            self.process_request(req);
        }

        // Handle button declicks
        for (id, elem) in &mut self.elements {
            let mut any = false;
            for state in &mut elem.state {
                if let State::Button { clicked } = state {
                    if *clicked {
                        *clicked = false;
                        any = true;
                    }
                }
            }

            if any {
                engine.send(UiUpdate {
                    id: *id,
                    state: elem.state.clone(),
                });
            }
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
        let mut needs_update = false;
        for (schema, state) in self.schema.iter().zip(&mut self.state) {
            needs_update |= show(ui, schema, state);
        }
        needs_update
    }
}


    */
