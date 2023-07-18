//! Basic graphical user interfacing
use cimvr_engine_interface::{dbg, pkg_namespace, prelude::*};
pub use egui;
use egui::{epaint, Shape};
use egui::{
    epaint::{ClippedShape, Primitive},
    ClippedPrimitive, FullOutput, InnerResponse, Mesh, Rect, TextureId, TexturesDelta, Ui,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub type GuiTabId = String;

/// Message sent from host GUI to plugin
#[derive(Message, Serialize, Deserialize, Debug)]
#[locality("Local")]
pub enum GuiConfigMessage {
    TabFullscreen(bool),
}

/// Message sent from host GUI to plugin
#[derive(Message, Serialize, Deserialize, Debug)]
#[locality("Local")]
pub struct GuiInputMessage {
    pub target: GuiTabId,
    pub raw_input: egui::RawInput,
}

/// Message sent from plugin to host GUI
#[derive(Message, Serialize, Deserialize)]
#[locality("Local")]
pub struct GuiOutputMessage {
    pub target: GuiTabId,
    pub output: Option<PartialOutput>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JankClippedMesh {
    pub clip: Rect,
    pub vertices: Vec<u8>,
    pub indices: Vec<u32>,
    pub texture_id: TextureId,
}

#[derive(Serialize, Deserialize)]
pub struct PartialOutput {
    pub shapes: Vec<JankClippedMesh>,
    pub textures_delta: TexturesDelta,
}

pub struct GuiTab {
    ctx: egui::Context,
    id: GuiTabId,
    texture_id_offset: u64,
}

impl GuiTab {
    pub fn new(io: &mut EngineIo, id: impl Into<GuiTabId>) -> Self {
        // Notify the system of the new element
        let id: GuiTabId = id.into();

        io.send(&GuiOutputMessage {
            target: id.clone(),
            output: None,
        });

        // Dumb hack
        //let texture_id_offset =
        //(const_hash(pkg_namespace!("TextureId")) % u128::from(u64::MAX)) as u64;
        let texture_id_offset = (io.random() % u128::from(u64::MAX)) as u64;

        Self {
            ctx: egui::Context::default(),
            id,
            texture_id_offset,
        }
    }

    pub fn show<R>(&mut self, io: &mut EngineIo, f: impl FnOnce(&mut Ui) -> R) {
        // Send dummy message (starts GUI)
        io.send(&GuiOutputMessage {
            target: self.id.clone(),
            output: Default::default(),
        });

        // Handle input messages
        let Some(msg) = io.inbox::<GuiInputMessage>().find(|msg| msg.target == self.id) else { return };

        // Process user's GUI
        let full_output = self.ctx.run(msg.raw_input, |ctx| {
            ctx.request_repaint();
            egui::CentralPanel::default().show(&self.ctx, f);
        });

        // Tesselate before serializing; faster
        let shapes = self.ctx.tessellate(full_output.shapes);
        let mut shapes: Vec<JankClippedMesh> = shapes.into_iter().map(|s| s.into()).collect();

        // Translate texture IDs
        // Delta messages
        let mut textures_delta = full_output.textures_delta;
        for (set, _) in &mut textures_delta.set {
            if let TextureId::Managed(id) = set {
                *id = id.wrapping_add(self.texture_id_offset);
            }
        }
        for free in &mut textures_delta.free {
            if let TextureId::Managed(id) = free {
                *id = id.wrapping_add(self.texture_id_offset);
            }
        }
        // Shapes
        for shape in &mut shapes {
            if let TextureId::Managed(id) = &mut shape.texture_id {
                *id = id.wrapping_add(self.texture_id_offset);
            }
        }

        // Send geometry to host
        io.send(&GuiOutputMessage {
            target: self.id.clone(),
            output: Some(PartialOutput {
                shapes,
                textures_delta,
            }),
        })
    }
}

impl From<ClippedPrimitive> for JankClippedMesh {
    fn from(value: ClippedPrimitive) -> Self {
        let Primitive::Mesh(mesh) = value.primitive else { panic!() };
        Self {
            clip: value.clip_rect,
            vertices: bytemuck::allocation::pod_collect_to_vec(&mesh.vertices),
            indices: mesh.indices,
            texture_id: mesh.texture_id,
        }
    }
}

impl Into<Mesh> for JankClippedMesh {
    fn into(self) -> Mesh {
        Mesh {
            indices: self.indices,
            vertices: bytemuck::allocation::pod_collect_to_vec(&self.vertices),
            texture_id: self.texture_id,
        }
    }
}

impl From<egui::epaint::Vertex> for crate::render::Vertex {
    fn from(value: egui::epaint::Vertex) -> Self {
        Self {
            pos: [value.pos.x, 0., value.pos.y],
            uvw: {
                let [r, g, b, _] = value.color.to_normalized_gamma_f32();
                [r, g, b]
            },
        }
    }
}

pub fn epaint_shape_to_cimvr_mesh(
    meters_per_point: f32,
    shape: Shape,
) -> Option<crate::render::Mesh> {
    let shapes = vec![ClippedShape(Rect::EVERYTHING, shape)];
    let mut tess = epaint::tessellate_shapes(240., Default::default(), [0, 0], vec![], shapes);

    match tess.pop().unwrap().primitive {
        Primitive::Mesh(mut mesh) => {
            // Rescale normals
            for v in &mut mesh.vertices {
                v.pos = (v.pos.to_vec2() * meters_per_point).to_pos2();
            }

            // Fix indexing direction
            for tri in mesh.indices.chunks_exact_mut(3) {
                let vert = |idx| mesh.vertices[tri[idx] as usize].pos;
                let base = vert(0);
                let a = vert(1) - base;
                let b = vert(2) - base;
                if a.x * b.y > a.y * b.x {
                    tri.swap(0, 2);
                }
            }

            Some(crate::render::Mesh {
                vertices: mesh.vertices.into_iter().map(|v| v.into()).collect(),
                indices: mesh.indices,
            })
        }
        _ => None,
    }
}
