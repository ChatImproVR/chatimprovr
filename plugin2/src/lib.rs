use cimvr_common::{
    nalgebra::{Point3, UnitQuaternion, Vector3},
    render::{Mesh, Primitive, Render, RenderData, RenderHandle, Vertex},
    FrameTime, Transform,
};
use cimvr_engine_interface::{make_app_state, prelude::*};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

struct ServerState;
struct ClientState;

make_app_state!(ClientState, ServerState);

const CUBE_HANDLE: RenderHandle = RenderHandle(3984203840);

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct MoveCube {
    pub r: f32,
}

impl UserState for ClientState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        io.send(&cube());
        Self
    }
}

impl UserState for ServerState {
    fn new(io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        // Cube mesh
        let cube_rdr = Render {
            id: CUBE_HANDLE,
            primitive: Primitive::Triangles,
            limit: None,
        };

        // Create central cube
        let cube_ent = io.create_entity();
        io.add_component(cube_ent, &Transform::default());
        io.add_component(cube_ent, &cube_rdr);
        io.add_component(cube_ent, &Synchronized);

        // Add cubes
        let n = 100_000;
        for i in 0..n {
            let i = i as f32 / n as f32;
            let cube_ent = io.create_entity();

            let r = i * TAU;

            io.add_component(cube_ent, &Transform::default());
            io.add_component(cube_ent, &cube_rdr);
            io.add_component(cube_ent, &Synchronized);
            io.add_component(cube_ent, &MoveCube { r });
        }

        // Schedule the system
        schedule.add_system(
            SystemDescriptor {
                stage: Stage::PreUpdate,
                subscriptions: vec![sub::<FrameTime>()],
                query: vec![
                    query::<Transform>(Access::Write),
                    query::<MoveCube>(Access::Read),
                ],
            },
            Self::cube_move,
        );

        Self
    }
}

impl ServerState {
    fn cube_move(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(FrameTime { time, .. }) = io.inbox_first() {
            for key in query.iter() {
                let mov = query.read::<MoveCube>(key);

                let theta = mov.r + time / 10.;
                let k = 3.;
                let v = (theta * k).sin() + 2.;

                let rad = 20. * v;

                let transf = Transform {
                    pos: Point3::new(theta.cos() * rad, 0., theta.sin() * rad),
                    orient: UnitQuaternion::face_towards(
                        &Vector3::new(
                            k * theta.cos() * (theta * k).cos() - theta.sin() * v,
                            0.,
                            k * theta.sin() * (theta * k).cos() + theta.cos() * v,
                        ),
                        &Vector3::y(),
                    ),
                };

                query.write::<Transform>(key, &transf);
            }
        }
    }
}

fn cube() -> RenderData {
    let vertices = vec![
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 1.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0, 1.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 1.0]),
        Vertex::new([-1.0, 1.0, 1.0], [1.0, 0.0, 1.0]),
    ];

    let indices = vec![
        3, 1, 0, 2, 1, 3, 2, 5, 1, 6, 5, 2, 6, 4, 5, 7, 4, 6, 7, 0, 4, 3, 0, 7, 7, 2, 3, 6, 2, 7,
        0, 5, 4, 1, 5, 0,
    ];

    RenderData {
        mesh: Mesh { vertices, indices },
        id: CUBE_HANDLE,
    }
}

impl Component for MoveCube {
    const ID: ComponentId = ComponentId {
        id: 0xC0BE,
        size: 4,
    };
}
