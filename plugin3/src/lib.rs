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
    moonch: UiHandle,
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

        sched.add_system(
            SystemDescriptor {
                stage: Stage::Update,
                subscriptions: vec![],
                query: vec![query::<Transform>(Access::Write)],
            },
            Self::move_up,
        );

        let schmeal = ui.add(
            io,
            "Schmeal".into(),
            vec![
                Schema::TextInput,
                Schema::Button {
                    text: "Hehe butt on".into(),
                },
                Schema::DragValue {
                    min: Some(-100.),
                    max: Some(420.0),
                },
                Schema::DragValue {
                    min: Some(-100.),
                    max: Some(420.0),
                },
            ],
            vec![
                State::TextInput { text: "no".into() },
                State::Button { clicked: false },
                State::DragValue { value: 0. },
                State::DragValue { value: 0. },
            ],
        );

        let moonch = ui.add(
            io,
            "Moonch".into(),
            vec![
                Schema::TextInput,
                Schema::Button {
                    text: "button,".into(),
                },
            ],
            vec![
                State::TextInput {
                    text: "Tegst inpud".into(),
                },
                State::Button { clicked: false },
            ],
        );

        for _ in 0..1000 {
            ui.add(
                io,
                "Moonch".into(),
                vec![
                    Schema::TextInput,
                    Schema::Button {
                        text: "button,".into(),
                    },
                ],
                vec![
                    State::TextInput {
                        text: "Tegst inpud".into(),
                    },
                    State::Button { clicked: false },
                ],
            );
        }

        Self {
            ui,
            schmeal,
            moonch,
        }
    }
}

impl ClientState {
    fn ui_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        self.ui.download(io);

        let ret = self.ui.read(self.schmeal);
        if ret[1] == (State::Button { clicked: true }) {
            dbg!(ret);
        }

        let ret = self.ui.read(self.moonch);
        if ret[1] == (State::Button { clicked: true }) {
            dbg!(ret);
        }

        /*
        if io.inbox::<UiUpdate>().next().is_some() {
            let val = self.ui.read(self.schmeal);
            //if val[1] == (State::Button { clicked: true }) {
            dbg!(val);
            //}
        }
        */
    }

    fn move_up(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let val = self.ui.read(self.schmeal);
        let State::DragValue { value } = val[2] else { panic!() };

        self.ui.modify(io, self.schmeal, |s| {
            s[3] = State::DragValue { value: value + 8.8 }
        });

        for key in query.iter() {
            query.modify::<Transform>(key, |v| v.pos.y += value);
        }
    }
}
