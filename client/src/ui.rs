use std::collections::HashMap;

use cimvr_common::ui::*;
use cimvr_engine::Engine;
use egui::{color_picker::color_edit_button_rgb, Context, DragValue, ScrollArea, TextEdit, Ui};

use crate::plugin_ui::PluginUi;

pub struct OverlayUi {
    plugin_ui: PluginUi,
}

impl OverlayUi {
    pub fn new(engine: &mut Engine) -> Self {
        Self {
            plugin_ui: PluginUi::new(engine),
        }
    }

    pub fn run(&mut self, ctx: &Context, engine: &mut Engine) {
        self.plugin_ui.run(ctx, engine);
    }

    pub fn update(&mut self, engine: &mut Engine) {
        self.plugin_ui.update(engine)
    }
}
