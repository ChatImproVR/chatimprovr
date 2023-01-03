use std::collections::HashMap;

use cimvr_common::ui::*;
use cimvr_engine::Engine;
use egui::{color_picker::color_edit_button_rgb, Context, DragValue, ScrollArea, TextEdit, Ui};

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
            ScrollArea::vertical().show(ui, |ui| {
                for (id, elem) in self.elements.iter_mut() {
                    ui.label(&elem.name);
                    if elem.show(ui) {
                        engine.send(UiUpdate {
                            id: *id,
                            state: elem.state.clone(),
                        });
                    }
                    ui.add_space(10.);
                }
            });
        });
    }

    pub fn update(&mut self, engine: &mut Engine) {
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

fn show(ui: &mut Ui, schema: &Schema, state: &mut State) -> bool {
    match (schema, state) {
        (Schema::Label { text }, State::Label) => ui.label(text).changed(),
        (Schema::TextInput, State::TextInput { text }) => {
            ui.add(TextEdit::singleline(text)).changed()
        }
        (Schema::Button { text }, State::Button { clicked }) => {
            *clicked = ui.button(text).clicked();
            *clicked
        }
        (Schema::DragValue { min, max }, State::DragValue { value }) => {
            let range = min.unwrap_or(f32::MIN)..=max.unwrap_or(f32::MAX);
            let dv = DragValue::new(value).clamp_range(range);
            ui.add(dv).changed()
        }
        (Schema::ColorPicker, State::ColorPicker { rgb }) => {
            color_edit_button_rgb(ui, rgb).changed()
        }
        (schema, state) => {
            log::error!(
                "Invalid UI schema and state combo: {:?} {:?}",
                schema,
                state
            );
            false
        }
    }
}
