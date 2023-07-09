use cimvr_common::{ui::GuiTab, Transform};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*, println, FrameTime};

struct ClientState {
    tab: GuiTab,
}

make_app_state!(ClientState, DummyUserState);

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched.add_system(Self::update_ui).build();

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
        });
    }
}
