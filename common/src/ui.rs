//! Basic graphical user interfacing
use cimvr_engine_interface::{pkg_namespace, prelude::*};
use egui::{InnerResponse, Ui};
use serde::{Deserialize, Serialize};

pub struct GuiTab {}

impl GuiTab {
    pub fn new(name: &str) -> Self {
        Self {}
    }

    pub fn show<R>(&mut self, io: &mut EngineIo, ui: impl FnOnce(&mut Ui) -> R) {}
}
