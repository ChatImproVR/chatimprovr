use cimvr_common::{
    ui::{egui, GuiInputMessage, GuiTab},
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*, println, FrameTime};

struct ClientState {
    tab: GuiTab,
}

make_app_state!(ClientState, DummyUserState);

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched
            .add_system(Self::update_ui)
            .subscribe::<GuiInputMessage>()
            .build();

        let tab = GuiTab::new(io, pkg_namespace!("MyTab"));

        Self { tab }
    }
}

impl ClientState {
    fn update_ui(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        self.tab.show(io, |ui| {
            if ui.button("Ohh yeahhh").clicked() {
                println!("I've been clicked!");
            }

            use egui::plot::{Line, Plot, PlotPoints};
            let sin: PlotPoints = (0..1000)
                .map(|i| {
                    let x = i as f64 * 0.01;
                    [x, x.sin()]
                })
                .collect();
            let line = Line::new(sin);
            Plot::new("my_plot")
                .view_aspect(2.0)
                .show(ui, |plot_ui| plot_ui.line(line));
        });
    }
}
