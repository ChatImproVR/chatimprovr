use cimvr_common::{
    glam::Vec3,
    render::{CameraComponent, Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    utils::client_tracker::{Action, ClientTracker},
    vr::VrUpdate,
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*, FrameTime};

use serde::{Deserialize, Serialize};

struct ServerState {
    tracker: ClientTracker,
}

struct ClientState {
    animation: SkeletonAnimator,
}

make_app_state!(ClientState, ServerState);

const CUBE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Cube"));
const SKELETON_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Skeleton"));

/// Request a server-side update to an avatar from the client side
#[derive(Message, Serialize, Deserialize, Clone)]
#[locality("Remote")]
pub struct AvatarUpdate {
    pub skeleton: Skeleton,
}

/// Informs a client which ID it has
#[derive(Message, Serialize, Deserialize, Clone)]
#[locality("Remote")]
pub struct ClientIdMessage(ClientId);

/// Associates an entity server-side with a client ID
#[derive(Component, Serialize, Deserialize, Clone, Default, Copy)]
pub struct AvatarComponent(ClientId);

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        io.send(&UploadMesh {
            mesh: cube(),
            id: CUBE_HANDLE,
        });

        io.create_entity()
            .add_component(Render::new(SKELETON_HANDLE).primitive(Primitive::Lines))
            .add_component(Transform::new())
            .build();

        sched
            .add_system(Self::update)
            .subscribe::<VrUpdate>()
            .subscribe::<ClientIdMessage>()
            .query(
                "Camera",
                Query::new()
                    .intersect::<CameraComponent>(Access::Read)
                    .intersect::<Transform>(Access::Read),
            )
            .query(
                "Avatars",
                Query::new().intersect::<AvatarComponent>(Access::Read),
            )
            .build();

        Self {
            animation: SkeletonAnimator::new(),
        }
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let Some(FrameTime { delta, .. }) = io.inbox_first() else { return };

        // Get the camera position
        let Some(camera_entity) = query.iter("Camera").next() else { return };

        let mut eyeball = query.read::<Transform>(camera_entity);

        // Or, if in VR mode, the camera component's position corresponds to the floor.
        // Use this and the VR update to find the world position of the headset
        if let Some(update) = io.inbox_first::<VrUpdate>() {
            eyeball = update.headset.left.transf * eyeball;
        }

        let skele_input = SkeletonAnimatorInputs {
            eyeball,
            left_hand: None,
            right_hand: None,
            speed: 0.,
            dt: delta,
        };
        let skeleton = self.animation.update(&skele_input);

        // Send to server
        io.send(&AvatarUpdate { skeleton });

        // Delete our own avatar from the scene...
        if let Some(ClientIdMessage(client_id)) = io.inbox_first() {
            for entity in query.iter("Avatars") {
                let AvatarComponent(other_client_id) = query.read(entity);
                if other_client_id == client_id {
                    io.remove_entity(entity);
                }
            }
        }
    }

    fn animation(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        /*
        let input = SkeletonAnimatorInputs {
            eyeball:
        };

        io.send(&UploadMesh {
            mesh: skeleton_mesh(&self.animation.update(&skele_input), [1.; 3]),
            id: SKELETON_HANDLE,
        });
        */
    }
}

impl UserState for ServerState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched
            .add_system(Self::update)
            .subscribe::<Connections>()
            .subscribe::<AvatarUpdate>()
            .query(
                "Clients",
                Query::new().intersect::<AvatarComponent>(Access::Read),
            )
            .build();

        Self {
            tracker: ClientTracker::new(),
        }
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Create and destroy avatars on connection/disconnection
        let Some(conns) = io.inbox_first::<Connections>() else { return };
        self.tracker.update(&conns, |conn, action| {
            match action {
                Action::Connected => {
                    // Add new entity on connection
                    io.create_entity()
                        .add_component(Transform::default())
                        .add_component(Render::new(CUBE_HANDLE).primitive(Primitive::Triangles))
                        .add_component(Synchronized)
                        .add_component(AvatarComponent(conn.id))
                        .build();
                }
                Action::Disconnected => {
                    // Remove disconnected avatars
                    for entity in query.iter("Clients") {
                        let AvatarComponent(other_client_id) = query.read(entity);
                        if other_client_id == conn.id {
                            io.remove_entity(entity);
                        }
                    }
                }
            }
        });

        // Update avatar content
        for (client, update) in io.inbox_clients::<AvatarUpdate>().collect::<Vec<_>>() {
            // Find corresponding entity, if any
            let entity = query.iter("Clients").find(|entity| {
                let AvatarComponent(other_client_id) = query.read(*entity);
                other_client_id == client
            });

            // Update properties of the client
            if let Some(entity) = entity {
                // TODO: Avatar colors!!
                io.add_component(entity, update.skeleton.head);
            }

            // Inform the client which one it is
            io.send_to_client(&ClientIdMessage(client), client);
        }
    }
}

fn cube() -> Mesh {
    let size = 0.25;

    let color = [1.; 3];
    let vertices = vec![
        Vertex::new([-size, -size, -size], color),
        Vertex::new([size, -size, -size], color),
        Vertex::new([size, size, -size], color),
        Vertex::new([-size, size, -size], color),
        Vertex::new([-size, -size, size], color),
        Vertex::new([size, -size, size], color),
        Vertex::new([size, size, size], color),
        Vertex::new([-size, size, size], color),
    ];

    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    Mesh { vertices, indices }
}

struct SkeletonAnimatorInputs {
    /// Position of the user's eye
    eyeball: Transform,
    /// Position of the user's left hand
    left_hand: Option<Transform>,
    /// Position of the user's left hand
    right_hand: Option<Transform>,
    /// Speed in m/s
    speed: f32,
    /// Time step (seconds)
    dt: f32,
}

struct SkeletonAnimator {
    animation_phase: f32,
}

#[derive(Serialize, Deserialize, Clone)]
struct Skeleton {
    head: Transform,
    shoulders: Vec3,
    left_hand: Vec3,
    right_hand: Vec3,
    butt: Vec3,
    left_foot: Vec3,
    right_foot: Vec3,
}

impl SkeletonAnimator {
    pub fn new() -> Self {
        Self {
            animation_phase: 0.,
        }
    }

    pub fn update(&mut self, input: &SkeletonAnimatorInputs) -> Skeleton {
        self.animation_phase += input.speed * input.dt;
        todo!()
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self {
            head: Transform::new().with_position(Vec3::new(0., 1.8, 0.)),
            shoulders: Vec3::new(0., 1.52, 0.),
            left_hand: Vec3::new(0.5, 1.52, -0.5),
            right_hand: Vec3::new(-0.5, 1.52, -0.5),
            butt: Vec3::new(0., 0.91, 0.),
            left_foot: Vec3::new(0.5, 0., 0.0),
            right_foot: Vec3::new(-0.5, 0., 0.0),
        }
    }
}

fn skeleton_mesh(skele: &Skeleton, color: [f32; 3]) -> Mesh {
    let mut mesh = Mesh::new();

    let mut add = |pos: Vec3| mesh.push_vertex(Vertex::new(pos.into(), color));
    let head = add(skele.head.pos);
    let butt = add(skele.butt);
    let shoulders = add(skele.shoulders);
    let left_hand = add(skele.left_hand);
    let right_hand = add(skele.right_hand);

    let left_foot = add(skele.left_foot);
    let right_foot = add(skele.right_foot);

    mesh.push_indices(&[
        // Spine
        head, shoulders, //.
        shoulders, butt, //.
        // Arms
        left_hand, shoulders, //.
        right_hand, shoulders, //.
        // Legs
        left_foot, butt, //.
        right_foot, butt, //.
    ]);

    mesh
}
