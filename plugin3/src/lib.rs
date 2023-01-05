use cimvr_common::{
    render::{Mesh, Render, RenderData, RenderExtra, RenderHandle, Vertex},
    ui::{Schema, State, UiHandle, UiStateHelper, UiUpdate},
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, prelude::*};
use serde::{Deserialize, Serialize};

make_app_state!(ClientState, DummyUserState);

/*
struct ServerState {
    rgb: [f32; 3],
    cube: EntityId,
}
*/

struct ClientState {
    ui: UiStateHelper,
    test_element: UiHandle,
    val: f32,
    rgb: [f32; 3],
}

/*
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeColor {
    rgb: [f32; 3],
}

impl Message for ChangeColor {
    const CHANNEL: ChannelId = ChannelId {
        id: 0x99999999999,
        locality: Locality::Remote,
    };
}

const CUBE_HANDLE: RenderHandle = RenderHandle(832094809);

impl UserState for ServerState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        let rgb = [1.; 3];

        io.send(&cube(rgb));

        let cube = io.create_entity();
        io.add_component(cube, &Transform::default());

        Self { rgb, cube }
    }
}
*/

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        let mut ui = UiStateHelper::new();

        sched.add_system(
            Self::ui_update,
            SystemDescriptor::new(Stage::Update).subscribe::<UiUpdate>(),
        );

        sched.add_system(
            Self::change_limit,
            SystemDescriptor::new(Stage::Update).query::<Render>(Access::Write),
        );

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

        Self {
            rgb: [1.; 3],
            ui,
            test_element,
            val: 0.,
        }
    }
}

impl ClientState {
    fn ui_update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        self.ui.download(io);

        let ret = self.ui.read(self.test_element);
        if ret[1] == (State::Button { clicked: true }) {
            dbg!(ret);
        }

        if let State::DragValue { value } = ret[2] {
            self.val = value;
        }

        if let State::ColorPicker { rgb } = ret[3] {
            self.rgb = rgb;
        }

        //if io.inbox::<UiUpdate>().next().is_some() {
        //let val = self.ui.read(self.test_element);
        //dbg!(val);
        //}
    }

    fn change_limit(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for key in query.iter() {
            if self.val > 0. {
                query.modify::<Render>(key, |r| r.limit = Some(self.val as u32));
            }

            let mut extra = [0.; 4 * 4];
            extra[..3].copy_from_slice(&self.rgb);
            extra[3] = 1.;
            io.add_component(key.entity(), &RenderExtra(extra));
        }
    }
}

/*
fn cube(color: [f32; 3]) -> RenderData {
let vertices = vec![
Vertex::new([-1.0, -1.0, -1.0], color),
Vertex::new([1.0, -1.0, -1.0], color),
Vertex::new([1.0, 1.0, -1.0], color),
Vertex::new([-1.0, 1.0, -1.0], color),
Vertex::new([-1.0, -1.0, 1.0], color),
Vertex::new([1.0, -1.0, 1.0], color),
Vertex::new([1.0, 1.0, 1.0], color),
Vertex::new([-1.0, 1.0, 1.0], color),
];

let indices = vec![
3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
0, 5, 4, 1, 5, 0,
];

RenderData {
mesh: Mesh { vertices, indices },
id: CUBE_HANDLE,
}
}
*/
