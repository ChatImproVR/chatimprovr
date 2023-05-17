use cimvr_common::{
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*};

struct ServerState;
struct ClientState;

make_app_state!(ClientState, ServerState);

const POSITIVE_LENGTH : f32 = 5.0;
const NEGATIVE_LENGTH: f32 = 2.0;

fn x_positive_line() -> Mesh {

    // List of vertex positions and colors
    let vertices = vec![
        Vertex::new([0., 0., 0.], [1.0, 0.0, 0.0]),
        Vertex::new([POSITIVE_LENGTH, 0., 0.], [1.0, 0.0, 0.0]),
    ];

    // Each 2 indices (indexing into vertices) define a line
    let indices = vec![
        0,1
    ];

    Mesh { vertices, indices }
}

fn x_negative_dotted_line() -> Mesh {

    let iteration = (NEGATIVE_LENGTH * 20.) as u32;

    let vertices = (0..iteration).map(|i| Vertex::new([-0.05 * i as f32, 0., 0.], [1.0, 0.0, 0.0])).collect::<Vec<Vertex>>();

    let indices = (0..iteration).collect();

    Mesh { vertices, indices}

}

fn y_positive_line() -> Mesh {

    // List of vertex positions and colors
    let vertices = vec![
        Vertex::new([0., 0., 0.], [0.0, 1.0, 0.0]),
        Vertex::new([0., POSITIVE_LENGTH, 0.], [0.0, 1.0, 0.0]),
    ];

    // Each 2 indices (indexing into vertices) define a line
    let indices = vec![
        0,1
    ];

    Mesh { vertices, indices }
}

fn y_negative_dotted_line() -> Mesh {

    let iteration = (NEGATIVE_LENGTH * 20.) as u32;

    let vertices = (0..iteration).map(|i| Vertex::new([0., -0.05 * i as f32, 0.], [0.0, 1.0, 0.0])).collect::<Vec<Vertex>>();

    let indices = (0..iteration).collect();

    Mesh { vertices, indices}
}

fn z_positive_line() -> Mesh {

    // List of vertex positions and colors
    let vertices = vec![
        Vertex::new([0., 0., 0.], [0.0, 0.0, 1.0]),
        Vertex::new([0., 0., POSITIVE_LENGTH], [0.0, 0.0, 1.0]),
    ];

    // Each 2 indices (indexing into vertices) define a line
    let indices = vec![
        0,1
    ];

    Mesh { vertices, indices }
}

fn z_negative_dotted_line() -> Mesh {

    let iteration = (NEGATIVE_LENGTH * 20.) as u32;

    let vertices = (0..iteration).map(|i| Vertex::new([0., 0., -0.05 * i as f32], [0.0, 0.0, 1.0])).collect::<Vec<Vertex>>();

    let indices = (0..iteration).collect();

    Mesh { vertices, indices}
}

/// This handle uniquely identifies the mesh data between all clients, and the server.
/// When the server copies the ECS data to the clients, they immediately know which mesh to render!
///
/// Note how we've used pkg_namespace!() to ensure that the name is closer to universally unique
const X_POSITIVE: MeshHandle = MeshHandle::new(pkg_namespace!("X_positive"));
const X_NEGATIVE: MeshHandle = MeshHandle::new(pkg_namespace!("X_negative"));
const Y_POSITIVE: MeshHandle = MeshHandle::new(pkg_namespace!("Y_positive"));
const Y_NEGATIVE: MeshHandle = MeshHandle::new(pkg_namespace!("Y_negative"));
const Z_POSITIVE: MeshHandle = MeshHandle::new(pkg_namespace!("Z_positive"));
const Z_NEGATIVE: MeshHandle = MeshHandle::new(pkg_namespace!("Z_negative"));


impl UserState for ClientState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        // Make the cube mesh available to the rendering engine
        // This defines the CUBE_HANDLE id to refer to the mesh we get from cube()
        io.send(&UploadMesh {
            mesh: x_positive_line(),
            id: X_POSITIVE,
        });

        io.send(&UploadMesh {
            mesh: x_negative_dotted_line(),
            id: X_NEGATIVE,
        });

        io.send(&UploadMesh {
            mesh: y_positive_line(),
            id: Y_POSITIVE,
        });

        io.send(&UploadMesh {
            mesh: y_negative_dotted_line(),
            id: Y_NEGATIVE,
        });

        io.send(&UploadMesh {
            mesh: z_positive_line(),
            id: Z_POSITIVE,
        });

        io.send(&UploadMesh {
            mesh: z_negative_dotted_line(),
            id: Z_NEGATIVE,
        });

        Self
    }
}

impl UserState for ServerState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        // Create an entity
        io.create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(X_POSITIVE).primitive(Primitive::Lines))
            .add_component(Synchronized)
            .build();

        io.create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(X_NEGATIVE).primitive(Primitive::Lines))
            .add_component(Synchronized)
            .build();

        io.create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(Y_POSITIVE).primitive(Primitive::Lines))
            .add_component(Synchronized)
            .build();

        io.create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(Y_NEGATIVE).primitive(Primitive::Lines))
            .add_component(Synchronized)
            .build();

        io.create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(Z_POSITIVE).primitive(Primitive::Lines))
            .add_component(Synchronized)
            .build();

        io.create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(Z_NEGATIVE).primitive(Primitive::Lines))
            .add_component(Synchronized)
            .build();

        Self
    }
}