//! Basic graphical user interfacing
use cimvr_engine_interface::{pkg_namespace, prelude::*};
use egui::{InnerResponse, Ui};
use serde::{Deserialize, Serialize};

pub type GuiTabId = String;

/// Message sent from host GUI to plugin
#[derive(Message, Serialize, Deserialize)]
#[locality("Local")]
pub struct GuiInputMessage {
    target: GuiTabId,
    raw_input: egui::RawInput,
}

/// Message sent from plugin to host GUI
#[derive(Message, Serialize, Deserialize)]
#[locality("Local")]
pub struct GuioutputMessage {
    target: GuiTabId,
    output: egui::FullOutput,
}

pub struct GuiTab {
    ctx: egui::Context,
    id: GuiTabId,
}

impl GuiTab {
    pub fn new(id: impl Into<GuiTabId>) -> Self {
        Self {
            ctx: egui::Context::default(),
            id: id.into(),
        }
    }

    pub fn show<R>(&mut self, io: &mut EngineIo, f: impl FnOnce(&mut Ui) -> R) {
        // Handle input messages
        let Some(msg) = io.inbox::<GuiInputMessage>().find(|msg| msg.target == self.id) else { return };
        let full_output = self.ctx.run(msg.raw_input, |ctx| {
            ctx.request_repaint();
            egui::CentralPanel::default().show(&self.ctx, f);
        });
        io.send(&GuioutputMessage {
            target: self.id.clone(),
            output: full_output,
        })
    }
}
