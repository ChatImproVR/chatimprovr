//! Basic graphical user interfacing
use std::time::Duration;

use cimvr_engine_interface::{pkg_namespace, prelude::*};
use egui::{epaint::ClippedShape, FullOutput, InnerResponse, TexturesDelta, Ui};
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
    pub repaint_after: Duration,
    pub textures_delta: TexturesDelta,
    pub shapes: Vec<ClippedShape>,
}

impl Into<FullOutput> for PartialOutput {
    fn into(self) -> FullOutput {
        let PartialOutput {
            repaint_after,
            textures_delta,
            shapes,
        } = self;
        FullOutput {
            platform_output: Default::default(),
            repaint_after,
            textures_delta,
            shapes,
        }
    }
}

impl From<FullOutput> for PartialOutput {
    fn from(value: FullOutput) -> Self {
        let FullOutput {
            repaint_after,
            textures_delta,
            shapes,
            ..
        } = value;
        PartialOutput {
            repaint_after,
            textures_delta,
            shapes,
        }
    }
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
        io.send(&GuiOutputMessage {
            target: self.id.clone(),
            output: Default::default(),
        });

        // Handle input messages
        let Some(msg) = io.inbox::<GuiInputMessage>().find(|msg| msg.target == self.id) else { return };

        let full_output = self.ctx.run(msg.raw_input, |ctx| {
            ctx.request_repaint();
            egui::CentralPanel::default().show(&self.ctx, f);
        });

        io.send(&GuiOutputMessage {
            target: self.id.clone(),
            output: Some(full_output.into()),
        })
    }
}
