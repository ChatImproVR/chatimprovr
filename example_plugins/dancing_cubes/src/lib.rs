use cimvr_common::{
    glam::{Mat3, Quat, Vec3},
    render::{
        Mesh, MeshHandle, Primitive, Render, ShaderHandle, ShaderSource, UploadMesh, Vertex,
        DEFAULT_VERTEX_SHADER,
    },
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*, FrameTime};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

struct ServerState;
struct ClientState;

make_app_state!(ClientState, ServerState);

#[derive(Component, Serialize, Deserialize, Default, Clone, Copy)]
pub struct MoveCube {
    pub r: f32,
}

impl UserState for ClientState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        io.send(&cube());
        io.send(&cube_shader());

        Self
    }
}

impl UserState for ServerState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Schedule the systems
        sched
            .add_system(Self::cube_move)
            .subscribe::<FrameTime>()
            .query::<Transform>(Access::Write)
            .query::<MoveCube>(Access::Read)
            .build();

        sched
            .add_system(Self::startup)
            .stage(Stage::PostInit)
            .query::<MoveCube>(Access::Read)
            .build();

        Self
    }
}

impl ServerState {
    fn startup(&mut self, io: &mut EngineIo, query: &mut QueryResult) {
        for k in query.iter() {
            io.remove_entity(k);
        }

        // Cube mesh
        let cube_rdr = Render::new(CUBE_HANDLE)
            .primitive(Primitive::Lines)
            .shader(CUBE_SHADER);

        // Create central cube
        io.create_entity()
            .add_component(Transform::default())
            .add_component(cube_rdr)
            .add_component(Synchronized)
            .build();

        // Add cubes
        let n = 30;
        for i in 0..n {
            let i = i as f32 / n as f32;

            let r = i * TAU;

            io.create_entity()
                .add_component(Transform::default())
                .add_component(cube_rdr)
                .add_component(Synchronized)
                .add_component(MoveCube { r })
                .build();
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
                    pos: Vec3::new(theta.cos() * rad, rad.cos() * 1., theta.sin() * rad),
                    orient: face_towards(
                        Vec3::new(
                            k * theta.cos() * (theta * k).cos() - theta.sin() * v,
                            -rad.sin() * 1.,
                            k * theta.sin() * (theta * k).cos() + theta.cos() * v,
                        ),
                        Vec3::Y,
                    ),
                };

                query.write::<Transform>(key, &transf);
            }
        }
    }
}

// Note that these can share a name because they have different types!
const CUBE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Cube"));
const CUBE_SHADER: ShaderHandle = ShaderHandle::new(pkg_namespace!("Cube"));

fn cube() -> UploadMesh {
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

    UploadMesh {
        mesh: Mesh { vertices, indices },
        id: CUBE_HANDLE,
    }
}

fn cube_shader() -> ShaderSource {
    let fragment_src = "
#version 330
precision mediump float;

in vec4 f_color;

out vec4 out_color;

void main() {
    vec3 color = f_color.rgb;
    out_color = vec4(color, 1.);
}"
    .into();
    ShaderSource {
        vertex_src: DEFAULT_VERTEX_SHADER.to_string(),
        fragment_src,
        id: CUBE_SHADER,
    }
}

// TODO: Add a PR to glam?
fn face_towards(dir: Vec3, up: Vec3) -> Quat {
    let zaxis = dir.normalize();
    let xaxis = up.cross(zaxis).normalize();
    let yaxis = zaxis.cross(xaxis).normalize();

    let mat = Mat3::from_cols(xaxis, yaxis, zaxis);

    Quat::from_mat3(&mat)
}
