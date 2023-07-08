use cimvr_common::{
    glam::{Mat3, Quat, Vec3},
    render::{
        Mesh, MeshHandle, Primitive, Render, ShaderHandle, ShaderSource, UploadMesh, Vertex,
        DEFAULT_VERTEX_SHADER,
    },
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*, println, FrameTime};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

struct ServerState;
struct ClientState;

make_app_state!(ClientState, DummyUserState);

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched.add_system(Self::update).build();

        Self
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        println!("UwU");
    }
}
