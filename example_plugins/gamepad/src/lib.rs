use std::collections::HashMap;

use cimvr_common::{
    gamepad::{Axis, GamepadState},
    glam::{EulerRot, Quat, Vec3},
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

struct ClientState;

#[derive(Message, Serialize, Deserialize, Debug)]
#[locality("Remote")]
struct AxisMessage {
    axis: f32,
}

make_app_state!(ClientState, ServerState);

const CUBE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Cube"));

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Make the cube mesh available to the rendering engine
        io.send(&UploadMesh {
            mesh: cube(),
            id: CUBE_HANDLE,
        });

        sched
            .add_system(Self::update)
            .subscribe::<GamepadState>()
            .build();

        Self
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        println!("Update");
        if let Some(GamepadState(gamepads)) = io.inbox_first::<GamepadState>() {
            if let Some(gamepad) = gamepads.get(0) {
                let axis = gamepad.axes[&Axis::LeftStickX];
                io.send(&AxisMessage { axis });
            }
        }
    }
}

#[derive(Component, Serialize, Deserialize, Default, Copy, Clone, Debug)]
struct SpinningCube(ClientId);

struct ServerState;

impl UserState for ServerState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched
            .add_system(Self::update)
            .query::<SpinningCube>(Access::Read)
            .query::<Transform>(Access::Write)
            .subscribe::<AxisMessage>()
            .subscribe::<Connections>()
            .build();

        Self
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let Some(conns) = io.inbox_first::<Connections>() else { return };

        // Find spinning cubes by their client ids
        let mut client_to_entity = HashMap::new();

        // For each entity mapping to a client that we know about, store the mapping
        // client -> entity
        // If the entity exists but the client doesn't, remove the entity
        for key in query.iter() {
            let SpinningCube(client_id) = query.read::<SpinningCube>(key);
            if conns.clients.iter().find(|c| c.id == client_id).is_some() {
                client_to_entity.insert(client_id, key);
            } else {
                io.remove_entity(key);
            }
        }

        // For each update message
        for (client_id, msg) in io.inbox_clients::<AxisMessage>().collect::<Vec<_>>() {
            if let Some(entity) = client_to_entity.get(&client_id) {
                // If the client already has a cube, update it's position
                let ClientId(number) = client_id;
                let transf = Transform {
                    orient: Quat::from_euler(EulerRot::XYZ, 0., msg.axis, 0.),
                    pos: Vec3::new(number as f32 * 1.5, 0., 0.),
                };
                io.add_component(*entity, transf);
            } else {
                // Otherwise create a new cube
                let cube_rdr = Render::new(CUBE_HANDLE).primitive(Primitive::Triangles);

                let ent = io
                    .create_entity()
                    .add_component(Transform::default())
                    .add_component(cube_rdr)
                    .add_component(Synchronized)
                    .add_component(SpinningCube(client_id))
                    .build();

                // Add the entity to the list so it appears we don't add anything twice
                client_to_entity.insert(client_id, ent);
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

pub fn from_euler_angles(roll: f32, pitch: f32, yaw: f32) -> Quat {
    let (sr, cr) = (roll * 0.5).sin_cos();
    let (sp, cp) = (pitch * 0.5).sin_cos();
    let (sy, cy) = (yaw * 0.5).sin_cos();

    Quat::from_xyzw(
        cr.clone() * cp.clone() * cy.clone() + sr.clone() * sp.clone() * sy.clone(),
        sr.clone() * cp.clone() * cy.clone() - cr.clone() * sp.clone() * sy.clone(),
        cr.clone() * sp.clone() * cy.clone() + sr.clone() * cp.clone() * sy.clone(),
        cr * cp * sy - sr * sp * cy,
    )
}
