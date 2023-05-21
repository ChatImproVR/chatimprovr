use cimvr_common::{
    render::{MeshHandle, Primitive, Render, UploadMesh},
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*};

struct ServerState;
struct ClientState;

make_app_state!(ClientState, ServerState);

const AXES_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Axes"));

impl UserState for ClientState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        let mesh = obj_reader::obj::obj_lines_to_mesh(include_str!("assets/axes.obj"));
        io.send(&UploadMesh {
            mesh,
            id: AXES_HANDLE,
        });

        Self
    }
}

impl UserState for ServerState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        let _cube_ent = io
            .create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(AXES_HANDLE).primitive(Primitive::Triangles))
            .add_component(Synchronized)
            .build();

        Self
    }
}
