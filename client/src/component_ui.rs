use cimvr_engine::{
    interface::{
        kobble::{DynamicValue, Schema, SchemaDeserializer},
        prelude::{Access, ComponentId, EntityId, QueryComponent},
        serial::{deserialize, serialize, serialize_into},
        ComponentSchema,
    },
    Engine,
};
use egui::{Context, DragValue, ScrollArea, Ui};
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
                let marker = if has_id { "[-] " } else { "" };
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
                        let Some(data) = engine.ecs().get_raw(entity, component) else { continue };

                        SchemaDeserializer::set_schema(schema);
                        if let Ok(SchemaDeserializer(mut dynamic)) =
                            deserialize(std::io::Cursor::new(data))
                        {
                            ui.label(format!("{}", component.id));

                            if editor(&mut dynamic, ui) {
                                let Some(data) = engine.ecs().get_mut(entity, component) else { continue };
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

fn editor(value: &mut DynamicValue, ui: &mut Ui) -> bool {
    match value {
        DynamicValue::Unit => false,
        DynamicValue::Bool(b) => ui.checkbox(b, "").clicked(),
        DynamicValue::String(s) | DynamicValue::UnitStruct(s) => {
            ui.label(s.clone());
            false
        }
        DynamicValue::NewtypeStruct(name, field) => {
            ui.horizontal(|ui| {
                ui.label(name.clone());
                editor(field, ui)
            })
            .inner
        }
        DynamicValue::Tuple(fields) => {
            let mut changed = false;
            for field_val in fields {
                ui.horizontal(|ui| {
                    changed |= editor(field_val, ui);
                });
            }
            changed
        }
        DynamicValue::TupleStruct(name, fields) => {
            if name == "Mat4" {
                return edit_matrix(value, ui);
            }

            if matches!(name.as_str(), "Vec3" | "Vec4" | "Quat") {
                return edit_vector(value, ui);
            }

            ui.label(name.clone());
            let mut changed = false;
            for field_val in fields {
                ui.horizontal(|ui| {
                    changed |= editor(field_val, ui);
                });
            }
            changed
        }
        DynamicValue::Struct { name, fields } => {
            ui.label(name.clone());
            let mut changed = false;
            for (name, field_val) in fields {
                ui.horizontal(|ui| {
                    ui.label(name.clone());
                    changed |= editor(field_val, ui);
                });
            }
            changed
        }
        DynamicValue::Enum(schema, sel_idx) => {
            let mut changed = false;
            ui.horizontal(|ui| {
                for (idx, variant) in schema.variants.iter().enumerate() {
                    let clicked = ui.radio(idx == *sel_idx as usize, variant).clicked();
                    changed |= clicked;
                    if clicked {
                        *sel_idx = idx as u32;
                    }
                }
            });
            changed
        }
        DynamicValue::I8(v) => ui.add(DragValue::new(v).speed(0.1)).changed(),
        DynamicValue::U8(v) => ui.add(DragValue::new(v).speed(0.1)).changed(),
        DynamicValue::I16(v) => ui.add(DragValue::new(v).speed(0.1)).changed(),
        DynamicValue::U16(v) => ui.add(DragValue::new(v).speed(0.1)).changed(),
        DynamicValue::I32(v) => ui.add(DragValue::new(v).speed(0.1)).changed(),
        DynamicValue::U32(v) => ui.add(DragValue::new(v).speed(0.1)).changed(),
        DynamicValue::I64(v) => ui.add(DragValue::new(v).speed(0.1)).changed(),
        DynamicValue::U64(v) => ui.add(DragValue::new(v).speed(0.1)).changed(),
        DynamicValue::I128(v) => {
            ui.label(format!("{}", v));
            false
        }
        DynamicValue::U128(v) => {
            ui.label(format!("{}", v));
            false
        }
        DynamicValue::Char(c) => {
            ui.label(format!("{}", c));
            false
        }
        DynamicValue::F32(v) => ui.add(DragValue::new(v).speed(0.1)).changed(),
        DynamicValue::F64(v) => ui.add(DragValue::new(v).speed(0.1)).changed(),
    }
}

fn edit_matrix(value: &mut DynamicValue, ui: &mut Ui) -> bool {
    if let DynamicValue::TupleStruct(_, fields) = value {
        let mut changed = false;
        ui.vertical(|ui| {
            for row in fields.chunks_exact_mut(4) {
                ui.horizontal(|ui| {
                    for col in row {
                        if let DynamicValue::F32(v) = col {
                            changed |= ui.add(DragValue::new(v).speed(0.1)).changed();
                        }
                    }
                });
            }
        });
        changed
    } else {
        false
    }
}

fn edit_vector(value: &mut DynamicValue, ui: &mut Ui) -> bool {
    if let DynamicValue::TupleStruct(_, fields) = value {
        let mut changed = false;
        ui.horizontal(|ui| {
            let names = ["x: ", "y: ", "z: ", "w: "];
            for (col, name) in fields.iter_mut().zip(names) {
                if let DynamicValue::F32(v) = col {
                    changed |= ui.add(DragValue::new(v).prefix(name).speed(0.1)).changed();
                }
            }
        });
        changed
    } else {
        false
    }
}
