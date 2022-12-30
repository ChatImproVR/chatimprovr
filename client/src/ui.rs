use egui::Context;

pub struct OverlayUi {
    test: [f32; 3],
}

impl OverlayUi {
    pub fn new() -> Self {
        Self { test: [0.; 3] }
    }

    pub fn run(&mut self, ctx: &Context) {
        egui::SidePanel::left("my_side_panel").show(ctx, |ui| {
            ui.heading("Hello World!");
            if ui.button("Quit").clicked() {
                dbg!("Nuh uh");
            }
            ui.color_edit_button_rgb(&mut self.test);
        });
    }
}
