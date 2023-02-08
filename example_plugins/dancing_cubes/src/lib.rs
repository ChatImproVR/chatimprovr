use cimvr_common::{
    nalgebra::{Point3, UnitQuaternion, Vector3},
    render::{
        Mesh, Primitive, Render, RenderData, RenderHandle, ShaderData, ShaderHandle, Vertex,
        DEFAULT_VERTEX_SHADER,
    },
    FrameTime, Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*, println};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

struct ServerState;
struct ClientState;

make_app_state!(ClientState, ServerState);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyMessage {
    hewwo: String,
}

impl Message for MyMessage {
    const CHANNEL: ChannelIdStatic = ChannelIdStatic {
        id: pkg_namespace!("MyMessage"),
        locality: Locality::Remote,
    };
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct MoveCube {
    pub r: f32,
}

impl UserState for ClientState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        io.send(&cube());
        io.send(&cube_shader());

        io.send(&MyMessage {
            hewwo: "I'm a client!".to_string(),
        });

        sched.add_system(
            Self::recv_server_msg,
            SystemDescriptor::new(Stage::Update).subscribe::<MyMessage>(),
        );

        Self
    }
}

impl ClientState {
    fn recv_server_msg(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        for msg in io.inbox::<MyMessage>() {
            dbg!(msg);
        }
    }
}

impl UserState for ServerState {
    fn new(_io: &mut EngineIo, schedule: &mut EngineSchedule<Self>) -> Self {
        println!("HEWWO");

        // Schedule the systems
        schedule.add_system(
            Self::cube_move,
            SystemDescriptor::new(Stage::Update)
                .subscribe::<FrameTime>()
                .query::<Transform>(Access::Write)
                .query::<MoveCube>(Access::Read),
        );

        schedule.add_system(
            Self::report_clients,
            SystemDescriptor::new(Stage::Update).subscribe::<MyMessage>(),
        );

        schedule.add_system(
            Self::startup,
            SystemDescriptor::new(Stage::PostInit).query::<MoveCube>(Access::Read),
        );

        Self
    }
}

impl ServerState {
    fn startup(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for k in query.iter() {
            io.remove_entity(k.entity());
        }

        // Cube mesh
        let cube_rdr = Render::new(CUBE_HANDLE)
            .primitive(Primitive::Lines)
            .shader(CUBE_SHADER);

        // Create central cube
        let cube_ent = io.create_entity();
        io.add_component(cube_ent, &Transform::default());
        io.add_component(cube_ent, &cube_rdr);
        io.add_component(cube_ent, &Synchronized);

        // Add cubes
        let n = 2000;
        for i in 0..n {
            let i = i as f32 / n as f32;
            let cube_ent = io.create_entity();

            let r = i * TAU;

            io.add_component(cube_ent, &Transform::default());
            io.add_component(cube_ent, &cube_rdr);
            io.add_component(cube_ent, &Synchronized);
            io.add_component(cube_ent, &MoveCube { r });
        }
    }

    fn report_clients(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        let msgs: Vec<_> = io.inbox_clients::<MyMessage>().collect();
        for (client, msg) in msgs {
            dbg!(&msg);
            io.send(&MyMessage {
                hewwo: format!(
                    "Haiiiii :3 I'm the server and you're {:?}. You said {}",
                    client, msg.hewwo
                ),
            });
        }
    }

    fn cube_move(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        if let Some(FrameTime { time, .. }) = io.inbox_first() {
            for key in query.iter() {
                let mov = query.read::<MoveCube>(key);

                let theta = mov.r + time / 10.;
                let k = 3.;
                let v = (theta * k).sin() + 2.;

                let rad = 20. * v;

                let transf = Transform {
                    pos: Point3::new(theta.cos() * rad, rad.cos() * 1., theta.sin() * rad),
                    orient: UnitQuaternion::face_towards(
                        &Vector3::new(
                            k * theta.cos() * (theta * k).cos() - theta.sin() * v,
                            -rad.sin() * 1.,
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

// Note that these can share a name because they have different types!
const CUBE_HANDLE: RenderHandle = RenderHandle::new(pkg_namespace!("Cube"));
const CUBE_SHADER: ShaderHandle = ShaderHandle::new(pkg_namespace!("Cube"));

fn cube() -> RenderData {
    let s = 0.25;
    let vertices = vec![
        Vertex::new([-s, -s, -s], [0.0, 1.0, 1.0]),
        Vertex::new([s, -s, -s], [1.0, 0.0, 1.0]),
        Vertex::new([s, s, -s], [1.0, 1.0, 0.0]),
        Vertex::new([-s, s, -s], [0.0, 1.0, 1.0]),
        Vertex::new([-s, -s, s], [1.0, 0.0, 1.0]),
        Vertex::new([s, -s, s], [1.0, 1.0, 0.0]),
        Vertex::new([s, s, s], [0.0, 1.0, 1.0]),
        Vertex::new([-s, s, s], [1.0, 0.0, 1.0]),
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

fn cube_shader() -> ShaderData {
    let fragment_src = "
#version 450
precision mediump float;

in vec4 f_color;

out vec4 out_color;

void main() {
    vec3 color = f_color.rgb;
    out_color = vec4(color, 1.);
}"
    .into();
    ShaderData {
        vertex_src: DEFAULT_VERTEX_SHADER.to_string(),
        fragment_src,
        id: CUBE_SHADER,
    }
}

impl Component for MoveCube {
    const ID: ComponentIdStatic = ComponentIdStatic {
        id: pkg_namespace!("MoveCube"),
        size: 4,
    };
}
