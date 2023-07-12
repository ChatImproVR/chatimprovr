use cimvr_common::{
    render::{
        Mesh, MeshHandle, Primitive, Render, RenderExtra, ShaderHandle, ShaderSource, UploadMesh,
        Vertex,
    },
    ui::GuiTab,
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*};

struct ClientState {
    shader_sources: ShaderSource,
    //fragment_edit_tab: GuiTab,
    //vertex_edit_tab: GuiTab,
}

make_app_state!(ClientState, DummyUserState);

/// This handle uniquely identifies the mesh data between all clients, and the server.
/// When the server copies the ECS data to the clients, they immediately know which mesh to render!
///
/// Note how we've used pkg_namespace!() to ensure that the name is closer to universally unique
const CUBE_HANDLE: MeshHandle = MeshHandle::new(pkg_namespace!("Cube"));

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
        // This defines the CUBE_HANDLE id to refer to the mesh we get from cube()
        io.send(&UploadMesh {
            mesh: dummy_mesh(),
            id: CUBE_HANDLE,
        });

        let id = ShaderHandle::new(pkg_namespace!("ShaderEditor"));

        io.create_entity()
            // Attach a Transform component (which defaults to the origin)
            .add_component(Transform::default())
            // Attach the Render component, which details how the object should be drawn
            // Note that we use CUBE_HANDLE here, to tell the rendering engine to draw the cube
            .add_component(
                Render::new(CUBE_HANDLE)
                    .primitive(Primitive::Triangles)
                    .shader(id),
            )
            // Add shader metadata
            .add_component(RenderExtra::default())
            .build();

        let shader_sources = ShaderSource {
            vertex_src: VERTEX_SRC.into(),
            fragment_src: FRAGMENT_SRC.into(),
            id,
        };

        io.send(&shader_sources);

        Self { shader_sources }
    }
}

/// Defines the mesh data fro a cube
fn dummy_mesh() -> Mesh {
    // List of vertex positions and colors
    let vertices = vec![Vertex::new([0.; 3], [0.; 3]); 3];

    // Each 3 indices (indexing into vertices) define a triangle
    let indices = vec![0, 1, 2];

    Mesh { vertices, indices }
}
