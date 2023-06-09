use cimvr_common::{
    render::{CameraComponent, Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    utils::client_tracker::{Action, ClientTracker},
    vr::VrUpdate,
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*};

use serde::{Deserialize, Serialize};

struct ServerState {
    tracker: ClientTracker,
}

struct ClientState;

make_app_state!(ClientState, ServerState);

const CUBE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Cube"));

/// Request a server-side update to an avatar from the client side
#[derive(Message, Serialize, Deserialize, Clone)]
#[locality("Remote")]
pub struct AvatarUpdate {
    pub head: Transform,
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

        Self
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        // Get the camera position
        let Some(camera_entity) = query.iter("Camera").next() else { return };

        let mut camera_tf = query.read::<Transform>(camera_entity);

        // Or, if in VR mode, the camera component's position corresponds to the floor.
        // Use this and the VR update to find the world position of the headset
        if let Some(update) = io.inbox_first::<VrUpdate>() {
            camera_tf = update.headset.left.transf * camera_tf;
        }

        // Send to server
        io.send(&AvatarUpdate { head: camera_tf });

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
                io.add_component(entity, update.head);
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
