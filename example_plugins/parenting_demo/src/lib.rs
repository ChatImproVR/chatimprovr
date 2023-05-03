use std::f32::consts::TAU;

use cimvr_common::{
    glam::{EulerRot, Quat, Vec3},
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*, FrameTime};
use parenting::ChildOf;
use serde::{Deserialize, Serialize};

struct ServerState;
struct ClientState;

make_app_state!(ClientState, ServerState);

const CUBE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Cube"));

impl UserState for ClientState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        io.send(&UploadMesh {
            mesh: cube(),
            id: CUBE_HANDLE,
        });

        Self
    }
}

/// Identifies the parent cube of all the other cubes
#[derive(Component, Copy, Clone, Debug, Default, Serialize, Deserialize)]
struct CentralCube;

impl UserState for ServerState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Create parent cube
        let parent_id = io
            .create_entity()
            .add_component(Transform::default())
            .add_component(Render::new(CUBE_HANDLE).primitive(Primitive::Triangles))
            .add_component(Synchronized)
            .add_component(CentralCube)
            .build();

        dbg!(parent_id);

        // Create child cubes
        let n = 30;
        for i in 0..n {
            let angle = TAU * i as f32 / n as f32;
            let _cube_ent = io
                .create_entity()
                .add_component(Transform::default())
                .add_component(Render::new(CUBE_HANDLE).primitive(Primitive::Triangles))
                .add_component(Synchronized)
                .add_component(ChildOf(
                    parent_id,
                    Transform::new()
                        .with_position(10. * Vec3::new(0., angle.cos(), angle.sin()))
                        .with_rotation(Quat::from_euler(EulerRot::XYZ, angle, 0., 0.)),
                ))
                .build();
        }

        sched
            .add_system(Self::update)
            .query(
                "MyCube",
                Query::new()
                    .intersect::<Transform>(Access::Write)
                    .intersect::<CentralCube>(Access::Read),
            )
            .subscribe::<FrameTime>()
            .build();

        Self
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        let FrameTime { time, .. } = io.inbox_first().unwrap();

        for entity in query.iter("MyCube") {
            query.modify(entity, |Transform { orient, .. }| {
                *orient = Quat::from_euler(EulerRot::XYZ, time / 10., 0., 0.)
            })
        }
    }
}

fn cube() -> Mesh {
    let size = 0.25;

    let mut vertices = vec![
        Vertex::new([-size, -size, -size], [0.0, 1.0, 1.0]),
        Vertex::new([size, -size, -size], [1.0, 0.0, 1.0]),
        Vertex::new([size, size, -size], [1.0, 1.0, 0.0]),
        Vertex::new([-size, size, -size], [0.0, 1.0, 1.0]),
        Vertex::new([-size, -size, size], [1.0, 0.0, 1.0]),
        Vertex::new([size, -size, size], [1.0, 1.0, 0.0]),
        Vertex::new([size, size, size], [0.0, 1.0, 1.0]),
        Vertex::new([-size, size, size], [1.0, 0.0, 1.0]),
    ];

    vertices.iter_mut().for_each(|v| v.pos[0] *= 30.);

    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    Mesh { vertices, indices }
}
