use cimvr_common::{
    glam::vec3,
    physics::{
        Action, ColliderHandle, ColliderShape, LocalColliderMsg, LocalRigidBodyMsg, Physics,
        PhysicsAction, RigidBody, RigidBodyBuilder, RigidBodyType,
    },
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
// Ignore this for now, all cubes are unit sized. UwU
const CUBE_COLLIDER: ColliderHandle = ColliderHandle::new(pkg_namespace!("CubeCollider"));
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
        io.send(&LocalColliderMsg {
            shape: ColliderShape::Cube(4.),
            // ColliderShape::Pill(4,5)
            // ColliderShape::Prism(3,4,5)
            handle: CUBE_COLLIDER,
        });

        // Create an entity
        for _ in 1..=5 {
            io.create_entity()
                // Attach a Transform component (which defaults to the origin)
                .add_component(Transform::default())
                .add_component(RigidBodyBuilder::new(RigidBodyType::Dynamic, CUBE_COLLIDER).build())
                // Attach the Render component, which details how the object should be drawn
                // Note that we use CUBE_HANDLE here, to tell the rendering engine to draw the cube
                .add_component(Render::new(CUBE_HANDLE).primitive(Primitive::Triangles))
                // Attach the Synchronized component, which will copy the object to clients
                .add_component(Synchronized)
                // And get the entity ID
                .build();
        }
        _sched
            .add_system(Self::foo)
            //.subscribe::<MoveCommand>()
            .query(
                "Cubes",
                Query::new()
                    // Do all rigidbodies and filter in the iter.
                    .intersect::<RigidBody>(Access::Read)
                    .intersect::<Transform>(Access::Write),
            )
            .build();
        Self
    }
}

impl ServerState {
    fn foo(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Check for movement commands
        for entity in query.iter("Cubes").filter(|x| {
            query
                .read::<RigidBody>(*x)
                .body_type
                .eq(&RigidBodyType::Dynamic)
        })
        // ugly, fix later
        {
            // Stuff in here will all be Dynamic bodies, cuz I wanna hit cubes UwU
            let phys_action = PhysicsAction::new(entity, Action::Force(vec3(0.0, 1000.0, 0.0)));
            io.send(&LocalRigidBodyMsg::new(phys_action));
        }
    }
}

/// Defines the mesh data fro a cube
fn cube() -> Mesh {
    // Size of the cube mesh
    let size = 0.25;

    // List of vertex positions and colors
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

    // Each 3 indices (indexing into vertices) define a triangle
    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    Mesh { vertices, indices }
}
