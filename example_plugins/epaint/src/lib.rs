use cimvr_common::{
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    ui::{
        egui::{Color32, Pos2, Shape, Stroke},
        epaint_shape_to_cimvr_mesh,
    },
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
        // This defines the CUBE_HANDLE id to refer to the mesh we get from cube()
        io.send(&UploadMesh {
            mesh: cube(),
            id: CUBE_HANDLE,
        });

        Self
    }
}

impl UserState for ServerState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        // Create an entity
        let _cube_ent = io
            .create_entity()
            // Attach a Transform component (which defaults to the origin)
            .add_component(Transform::default())
            // Attach the Render component, which details how the object should be drawn
            // Note that we use CUBE_HANDLE here, to tell the rendering engine to draw the cube
            .add_component(Render::new(CUBE_HANDLE).primitive(Primitive::Triangles))
            // Attach the Synchronized component, which will copy the object to clients
            .add_component(Synchronized)
            // And get the entity ID
            .build();

        Self
    }
}

/// Defines the mesh data fro a cube
fn cube() -> Mesh {
    let shapes = vec![
        Shape::line(
            vec![Pos2::new(-100., -100.), Pos2::new(100., 100.)],
            Stroke::new(5., Color32::RED),
        ),
        Shape::circle_filled(Pos2::new(0., 0.), 100., Color32::LIGHT_GREEN),
    ];

    epaint_shape_to_cimvr_mesh(1e-3, Shape::Vec(shapes)).unwrap()
}
