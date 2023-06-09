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
out vec4 f_color;

// https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
void main() {
    vec2 uv = vec2((gl_VertexID << 1) & 2, gl_VertexID & 2);
    f_color = vec4(uv, 0, 0);
    gl_Position = vec4(uv.xy * 2.0f + -1.0f, 0.0f, 1.0f);
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
            mesh: triangle(),
            id: CUBE_HANDLE,
        });

        io.send(&ShaderSource {
            vertex_src: VERTEX_SRC.into(),
            fragment_src: FRAGMENT_SRC.into(),
            id: CUBE_SHADER,
        });

        Self
    }
}

impl UserState for ServerState {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Define how the cube should be rendered
        let cube_rdr = Render::new(CUBE_HANDLE)
            .shader(CUBE_SHADER)
            .primitive(Primitive::Triangles);

        // Create one cube entity at the origin, and make it synchronize to clients
        let cube_ent = io
            .create_entity()
            .add_component(Transform::default())
            .add_component(cube_rdr)
            .add_component(Synchronized)
            .build();

        sched
            .add_system(Self::update)
            .subscribe::<FrameTime>()
            .build();

        Self { cube_ent }
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        let time = io.inbox_first::<FrameTime>().unwrap();

        let mut extra = [0.; 4 * 4];
        extra[0] = time.time;

        io.add_component(self.cube_ent, RenderExtra(extra));
        io.add_component(
            self.cube_ent,
            Transform::identity().with_position(Vec3::new(time.time.cos(), 0., 0.)),
        );
    }
}

/// Defines the mesh data fro a cube
fn triangle() -> Mesh {
    // These are just dummy inputs to get the vertex shader to execute for all 3 indices
    let vertices = vec![Vertex::new([0.; 3], [0.; 3]); 3];
    let indices = vec![0, 1, 2];

    Mesh { vertices, indices }
}
