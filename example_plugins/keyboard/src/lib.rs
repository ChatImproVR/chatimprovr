use cimvr_common::{
    desktop::{InputEvent, KeyCode},
    glam::Vec3,
    render::{Mesh, MeshHandle, Render, UploadMesh, Vertex},
    utils::input_helper::InputHelper,
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*, FrameTime};
use serde::{Deserialize, Serialize};

struct ServerState;
#[derive(Default)]
struct ClientState {
    input: InputHelper,
}

#[derive(Message, Serialize, Deserialize, Clone, Copy)]
#[locality("Remote")]
pub struct MoveCommand {
    pub distance: Vec3,
}

/// Component identifing the cube
#[derive(Component, Serialize, Deserialize, Default, Clone, Copy)]
pub struct CubeFlag;

make_app_state!(ClientState, ServerState);

/// This handle uniquely identifies the mesh data between all clients, and the server.
const CUBE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Cube"));

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Make the cube mesh available to the rendering engine
        io.send(&UploadMesh {
            mesh: cube(),
            id: CUBE_HANDLE,
        });

        // Add update system, and subscribe to needed channels
        sched
            .add_system(Self::update)
            .subscribe::<InputEvent>()
            .subscribe::<FrameTime>()
            .build();

        // SystemDescriptor::new(Stage::Update)
        //     .subscribe::<InputEvent>()
        //     .subscribe::<FrameTime>(),
        // Initialize state
        Self::default()
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        self.input.handle_input_events(io);
        // Read frame time (or bust!)
        let Some(frame_time) = io.inbox_first::<FrameTime>() else { return };

        // Handle input events
        // Decide which way the cube should move based on keypresses
        let mut move_vector = Vec3::ZERO;

        if self.input.key_held(KeyCode::W) {
            move_vector += Vec3::new(1., 0., 0.);
        }
        if self.input.key_held(KeyCode::A) {
            move_vector += Vec3::new(0., 0., -1.)
        }
        if self.input.key_held(KeyCode::S) {
            move_vector += Vec3::new(-1., 0., 0.)
        }
        if self.input.key_held(KeyCode::D) {
            move_vector += Vec3::new(0., 0., 1.)
        }

        // Send movement command to server
        if move_vector != Vec3::ZERO {
            let distance = move_vector.normalize() * frame_time.delta * 10.;

            let command = MoveCommand { distance };

            io.send(&command);
        }
    }
}

impl UserState for ServerState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Define how the cube should be rendered

        // Create one cube entity at the origin, and make it synchronize to clients
        io.create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(CUBE_HANDLE))
            .add_component(Synchronized)
            .add_component(CubeFlag)
            .build();

        // Create the Update system, which interprets movement commands and updates the transform
        // component on the object with CubeFlag
        sched
            .add_system(Self::update)
            .subscribe::<MoveCommand>()
            .query("Cubes")
            .intersect::<CubeFlag>(Access::Write)
            .intersect::<Transform>(Access::Write)
            .qcommit()
            .build();

        Self
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Check for movement commands
        if let Some(MoveCommand { distance }) = io.inbox_first() {
            // Update each object accordingly
            for key in query.iter("Cubes") {
                query.modify::<Transform>(key, |tf| {
                    tf.pos += distance;
                })
            }
        }
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
