use cimvr_common::ui::{Schema, State, UiHandle, UiStateHelper, UiUpdate};
use cimvr_engine_interface::{dbg, prelude::*};

use crate::ChangeColor;

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
                    text: "Test button".into(),
                },
                Schema::DragValue {
                    min: Some(-100.),
                    max: Some(420.0),
                },
                Schema::ColorPicker,
            ],
            vec![
                State::TextInput { text: "no".into() },
                State::Button { clicked: false },
                State::DragValue { value: 0. },
                State::ColorPicker { rgb: [1.; 3] },
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
            dbg!(ret);
        }

        if io.inbox::<UiUpdate>().next().is_some() {
            if let State::ColorPicker { rgb } = ret[3] {
                io.send(&ChangeColor { rgb });
            }
        }
    }
}
