use cimvr_common::{
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*};

struct ServerState;
struct ClientState;

make_app_state!(ClientState, ServerState);

const GRID_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Grid"));

impl UserState for ClientState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        io.send(&UploadMesh {
            mesh: grid_mesh(30, 0.1, [0.05; 3]),
            id: GRID_HANDLE,
        });

        Self
    }
}

impl UserState for ServerState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        let _cube_ent = io
            .create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(GRID_HANDLE).primitive(Primitive::Lines))
            .add_component(Synchronized)
            .build();

        Self
    }
}

pub fn grid_mesh(n: i32, scale: f32, color: [f32; 3]) -> Mesh {
    let mut m = Mesh::new();

    let width = n as f32 * scale;

    for i in -n..=n {
        let j = i as f32 * scale;

        let positions = [
            [j, 0.0, width],
            [j, 0.0, -width],
            [width, 0.0, j],
            [-width, 0.0, j],
        ];

        for pos in positions {
            let idx = m.push_vertex(Vertex::new(pos, color));
            m.indices.push(idx);
        }
    }

    m
}
