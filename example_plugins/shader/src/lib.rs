use cimvr_common::{
    glam::Vec3,
    render::{
        Mesh, MeshHandle, Primitive, Render, RenderExtra, ShaderHandle, ShaderSource, UploadMesh,
        Vertex,
    },
    Transform,
};
use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*, FrameTime};

struct ServerState {
    cube_ent: EntityId,
}

struct ClientState;

make_app_state!(ClientState, ServerState);

/// This handle uniquely identifies the mesh data between all clients, and the server.
/// When the server copies the ECS data to the clients, they immediately know which mesh to render!
///
/// Note how we've used pkg_namespace!() to ensure that the name is closer to universally unique
const CUBE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Cube"));
const CUBE_SHADER: ShaderHandle = ShaderHandle::new(pkg_namespace!("Cube"));

const VERTEX_SRC: &str = r#"
#version 450

uniform mat4 view;
uniform mat4 proj;
uniform mat4 transf;
uniform mat4 extra;

layout (location = 0) in vec3 pos;
layout (location = 1) in vec3 uvw;

out vec4 f_color;

void main() {
    gl_Position = proj * view * transf * vec4(pos, 1.);
    float k = extra[0][0];
    vec3 color = vec3(fract(k));
    gl_PointSize = 5.0;
    f_color = vec4(color, 1.);
}
"#;

const FRAGMENT_SRC: &str = r#"
#version 450
precision mediump float;

in vec4 f_color;

out vec4 out_color;

void main() {
    out_color = f_color;
}
"#;

impl UserState for ClientState {
    fn new(io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        // Make the cube mesh available to the rendering engine
        io.send(&UploadMesh {
            mesh: cube(),
            id: CUBE_HANDLE,
        });

        io.send(&ShaderSource {
            vertex_src: VERTEX_SRC.into(),
            fragment_src: FRAGMENT_SRC.into(),
            id: CUBE_SHADER,
        });

        _sched.add_system(
            Self::deleteme,
            SystemDescriptor::new(Stage::Update)
                .query::<RenderExtra>(Access::Read)
                .query::<Synchronized>(Access::Read),
        );

        Self
    }
}

impl ClientState {
    fn deleteme(&mut self, _io: &mut EngineIo, query: &mut QueryResult) {
        for key in query.iter() {
            let time = query.read::<RenderExtra>(key).0[0];
            dbg!((time, key));
        }
    }
}

impl UserState for ServerState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Define how the cube should be rendered
        let cube_rdr = Render::new(CUBE_HANDLE)
            .shader(CUBE_SHADER)
            .primitive(Primitive::Triangles);

        // Create one cube entity at the origin, and make it synchronize to clients
        let cube_ent = io.create_entity();
        io.add_component(cube_ent, &Transform::default());
        io.add_component(cube_ent, &cube_rdr);
        io.add_component(cube_ent, &Synchronized);
        sched.add_system(
            Self::update,
            SystemDescriptor::new(Stage::Update).subscribe::<FrameTime>(),
        );

        Self { cube_ent }
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        let time = io.inbox_first::<FrameTime>().unwrap();

        let mut extra = [0.; 4 * 4];
        extra[0] = time.time;

        io.add_component(self.cube_ent, &RenderExtra(extra));
        io.add_component(
            self.cube_ent,
            &Transform::identity().with_position(Vec3::new(time.time.cos(), 0., 0.)),
        );
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
