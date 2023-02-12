use cimvr_common::{
    render::{Mesh, MeshHandle, UploadMesh, Vertex},
    ui::{Schema, State, UiHandle, UiStateHelper, UiUpdate},
};
use cimvr_engine_interface::{dbg, pkg_namespace, prelude::*};

use crate::ChangeColor;

pub struct ClientState {
    ui: UiStateHelper,
    test_element: UiHandle,
}

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        let mut ui = UiStateHelper::new();

        io.send(&cube());

        sched.add_system(
            Self::ui_update,
            SystemDescriptor::new(Stage::Update).subscribe::<UiUpdate>(),
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

fn cube() -> UploadMesh {
    let vertices = vec![
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, 1.0, 1.0], [1.0, 0.0, 1.0]),
    ];

    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    UploadMesh {
        mesh: Mesh { vertices, indices },
        id: MeshHandle::new(pkg_namespace!("Cube")),
    }
}
