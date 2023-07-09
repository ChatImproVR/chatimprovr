//! Basic graphical user interfacing
use std::time::Duration;

use cimvr_engine_interface::{pkg_namespace, prelude::*};
pub use egui;
use egui::{epaint::ClippedShape, ClippedPrimitive, FullOutput, InnerResponse, TexturesDelta, Ui};
use serde::{Deserialize, Serialize};

pub type GuiTabId = String;

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

#[derive(Serialize, Deserialize)]
pub struct PartialOutput {
    pub shapes: Vec<ClippedPrimitive>,
}

pub struct GuiTab {
    ctx: egui::Context,
    id: GuiTabId,
}

impl GuiTab {
    pub fn new(io: &mut EngineIo, id: impl Into<GuiTabId>) -> Self {
        // Notify the system of the new element
        let id: GuiTabId = id.into();

        io.send(&GuiOutputMessage {
            target: id.clone(),
            output: None,
        });

        Self {
            ctx: egui::Context::default(),
            id,
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

        // Send geometry to host
        io.send(&GuiOutputMessage {
            target: self.id.clone(),
            output: Some(PartialOutput { shapes }),
        })
    }
}
