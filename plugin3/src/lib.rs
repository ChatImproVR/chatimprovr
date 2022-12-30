use cimvr_common::{
    nalgebra::{Point3, UnitQuaternion, Vector3},
    render::{Mesh, Primitive, Render, RenderData, RenderHandle, Vertex},
    ui::{Schema, State, UiHandle, UiStateHelper, UiUpdate},
    FrameTime, Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, prelude::*, println};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

struct ClientState {
    ui: UiStateHelper,
    schmeal: UiHandle,
}

make_app_state!(ClientState, DummyUserState);

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        let mut ui = UiStateHelper::new();

        sched.add_system(
            SystemDescriptor {
                stage: Stage::Update,
                subscriptions: vec![sub::<UiUpdate>()],
                query: vec![],
            },
            Self::ui_update,
        );

        let schmeal = ui.add(
            io,
            "Thing".into(),
            vec![
                Schema::Button {
                    text: "Schmeal".into(),
                },
                Schema::TextInput,
            ],
            vec![
                State::Button { clicked: false },
                State::TextInput { text: "no".into() },
            ],
        );

        Self { ui, schmeal }
    }
}

impl ClientState {
    fn ui_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        self.ui.download(io);

        dbg!(self.ui.read(self.schmeal));
    }
}
