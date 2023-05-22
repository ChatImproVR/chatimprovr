use cimvr_common::{
    ui::{Schema, State, UiHandle, UiStateHelper, UiUpdate},
    InterdimensionalTravelRequest,
};
use cimvr_engine_interface::{dbg, make_app_state, prelude::*};

pub struct ClientState {
    ui: UiStateHelper,
    test_element: UiHandle,
}

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        let mut ui = UiStateHelper::new();

        sched
            .add_system(Self::ui_update)
            .subscribe::<UiUpdate>()
            .build();

        let test_element = ui.add(
            io,
            "Properties".into(),
            vec![
                Schema::TextInput,
                Schema::Button {
                    text: "Connect".into(),
                },
            ],
            vec![
                State::TextInput {
                    text: "based.rs".into(),
                },
                State::Button { clicked: false },
            ],
        );

        Self { ui, test_element }
    }
}

impl ClientState {
    fn ui_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        self.ui.download(io);

        let ret = self.ui.read(self.test_element);
        if ret[1] == (State::Button { clicked: true }) {
            let State::TextInput { text } = &ret[0] else { panic!() };
            io.send(&InterdimensionalTravelRequest {
                address: text.into(),
            })
        }
    }
}

make_app_state!(ClientState, DummyUserState);
