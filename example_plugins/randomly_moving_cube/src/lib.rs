use cimvr_common::{
    glam::Vec3,
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*};

struct ServerState;
struct ClientState;

make_app_state!(ClientState, ServerState);

/// This handle uniquely identifies the mesh data between all clients, and the server.
/// When the server copies the ECS data to the clients, they immediately know which mesh to render!
///
/// Note how we've used pkg_namespace!() to ensure that the name is closer to universally unique
const CUBE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Cube"));

impl UserState for ClientState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        // Make the cube mesh available to the rendering engine
        io.send(&UploadMesh {
            mesh: cube(),
            id: CUBE_HANDLE,
        });

        Self
    }
}

impl UserState for ServerState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        // Define how the cube should be rendered
        let cube_rdr = Render::new(CUBE_HANDLE).primitive(Primitive::Triangles);

        // Random position.
        // TODO: This is pretty dirty, we should fix this!
        let mut rand = || {
            let mut v = [0; 4];
            v.copy_from_slice(&io.random().to_le_bytes()[..4]);
            let v = u32::from_le_bytes(v) as f32 / u32::MAX as f32;
            v * 3.
        };

        let pos = Vec3::new(rand(), rand(), rand());

        let cube_ent = io
            .create_entity()
            .add_component(Transform::default().with_position(pos))
            .add_component(cube_rdr)
            .add_component(Synchronized);
        // .add_component(Saved); // NOTE: Uncomment me!

        Self
    }
}

/// Defines the mesh data fro a cube
fn cube() -> Mesh {
    let size = 0.25;
    let vertices = vec![
        Vertex::new([-size, -size, -size], [0.0, 1.0, 1.0]),
        Vertex::new([size, -size, -size], [1.0, 0.0, 1.0]),
        Vertex::new([size, size, -size], [1.0, 1.0, 0.0]),
        Vertex::new([-size, size, -size], [0.0, 1.0, 1.0]),
        Vertex::new([-size, -size, size], [1.0, 0.0, 1.0]),
        Vertex::new([size, -size, size], [1.0, 1.0, 0.0]),
        Vertex::new([size, size, size], [0.0, 1.0, 1.0]),
        Vertex::new([-size, size, size], [1.0, 0.0, 1.0]),
    ];

    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    Mesh { vertices, indices }
}
