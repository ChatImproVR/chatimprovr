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
const SKELETONS_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Skeletons"));

/// Request a server-side update to an avatar from the client side
#[derive(Message, Serialize, Deserialize, Clone)]
#[locality("Remote")]
pub struct AvatarUpdate {
    skeleton: Skeleton,
}

/// Informs a client which ID it has
#[derive(Message, Serialize, Deserialize, Clone)]
#[locality("Remote")]
pub struct ClientIdMessage(ClientId);

/// Associates an entity server-side with a client ID
#[derive(Component, Serialize, Deserialize, Clone, Default, Copy)]
pub struct AvatarComponent(ClientId);

/// Data about a player's skeleton synchronized back to the client
#[derive(Component, Serialize, Deserialize, Clone, Default, Copy)]
pub struct AvatarSkeleton(Skeleton);

/// Flag marking an avatar's head
#[derive(Component, Serialize, Deserialize, Clone, Default, Copy)]
pub struct AvatarHead;

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        io.send(&UploadMesh {
            mesh: cube(),
            id: CUBE_HANDLE,
        });

        sched
            .add_system(Self::update)
            .subscribe::<VrUpdate>()
            .subscribe::<ClientIdMessage>()
            .subscribe::<FrameTime>()
            .query(
                "Camera",
                Query::new()
                    .intersect::<CameraComponent>(Access::Read)
                    .intersect::<Transform>(Access::Read),
            )
            .query(
                "Avatars",
                Query::new()
                    .intersect::<AvatarSkeleton>(Access::Read)
                    .intersect::<AvatarComponent>(Access::Read)
                    .intersect::<AvatarHead>(Access::Read),
            )
            .build();

        sched
            .add_system(Self::skeletons_update)
            .query(
                "Skeletons",
                Query::new().intersect::<AvatarSkeleton>(Access::Read),
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
            // TODO
            left_hand: None,
            right_hand: None,
            speed: 0.,
            dt: delta,
        };
        let skeleton = self.animation.update(&skele_input);

        // Send to server
        io.send(&AvatarUpdate { skeleton });

        // Delete our own avatar's head from the scene...
        if let Some(ClientIdMessage(client_id)) = io.inbox_first() {
            for entity in query.iter("Avatars") {
                let AvatarComponent(other_client_id) = query.read(entity);
                if other_client_id == client_id && query.has_component::<AvatarHead>(entity) {
                    io.remove_entity(entity);
                }
            }
        }
    }

    fn skeletons_update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let mut mesh = Mesh::new();
        for entity in query.iter("Skeletons") {
            let AvatarSkeleton(skeleton) = query.read(entity);
            skeleton_mesh(&mut mesh, &skeleton, [1.; 3]);
        }
        io.send(&UploadMesh {
            mesh,
            id: SKELETONS_HANDLE,
        });
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
                Query::new()
                    .intersect::<AvatarComponent>(Access::Read)
                    .intersect::<AvatarHead>(Access::Read)
                    .intersect::<AvatarSkeleton>(Access::Read),
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
                        .add_component(AvatarHead)
                        .add_component(Transform::default())
                        .add_component(Render::new(CUBE_HANDLE).primitive(Primitive::Triangles))
                        .add_component(Synchronized)
                        .add_component(AvatarComponent(conn.id))
                        .build();

                    io.create_entity()
                        .add_component(AvatarSkeleton(Skeleton::default()))
                        .add_component(Transform::new())
                        .add_component(Render::new(SKELETONS_HANDLE).primitive(Primitive::Lines))
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
            let head_entity = query.iter("Clients").find(|entity| {
                let AvatarComponent(other_client_id) = query.read(*entity);
                other_client_id == client && query.has_component::<AvatarHead>(*entity)
            });

            // Find corresponding entity, if any
            let skeleton_entity = query.iter("Clients").find(|entity| {
                let AvatarComponent(other_client_id) = query.read(*entity);
                other_client_id == client && query.has_component::<AvatarSkeleton>(*entity)
            });

            // Set head position
            if let Some(entity) = head_entity {
                io.add_component(entity, update.skeleton.head);
            }

            // Set skeleton
            if let Some(entity) = skeleton_entity {
                io.add_component(entity, AvatarSkeleton(update.skeleton));
            }

            // Inform the client which one it is
            io.send_to_client(&ClientIdMessage(client), client);
        }
    }
}

fn cube() -> Mesh {
    let size = 0.10;

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

#[derive(Serialize, Deserialize, Clone, Copy)]
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
        let mut skele = Skeleton::default();
        skele.head = input.eyeball;
        self.animation_phase += input.speed * input.dt;
        skele
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

fn skeleton_mesh(mesh: &mut Mesh, skele: &Skeleton, color: [f32; 3]) {
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
}
