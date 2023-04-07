use cimvr_common::{
    render::{Primitive, Render, UploadMesh, MeshHandle},
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*, println};

use crate::obj::obj_lines_to_mesh;

mod obj;

// All state associated with client-side behaviour
struct ClientState;

pub const SHIP_RDR: MeshHandle = MeshHandle::new(pkg_namespace!("Ship"));

impl UserState for ClientState {
    // Implement a constructor
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        //let mesh = obj_lines_to_mesh(include_str!("assets/ship.obj"));
        let mesh = obj_lines_to_mesh(include_str!("assets/dodecahedron.obj"));
        io.send(&UploadMesh { mesh, id: SHIP_RDR });

        // NOTE: We are using the println defined by cimvr_engine_interface here, NOT the standard library!
        cimvr_engine_interface::println!("This prints");
        std::println!("But this doesn't");

        Self
    }
}

// All state associated with server-side behaviour
struct ServerState;

impl UserState for ServerState {
    // Implement a constructor
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        let ent = io.create_entity().build();
        io.add_component(ent, &Transform::identity());
        io.add_component(ent, &Render::new(SHIP_RDR).primitive(Primitive::Triangles));
        io.add_component(ent, &Synchronized);
        
        println!("Hello, server!");
        Self
    }
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, ServerState);
