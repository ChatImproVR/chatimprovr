//! Types for interfacing with the Host's rendering engine
use bytemuck::{Pod, Zeroable};
use cimvr_engine_interface::{pkg_namespace, prelude::*};
use nalgebra::Matrix4;
use serde::{Deserialize, Serialize};

use crate::{make_handle, GenericHandle};

/// The default vertex shader
pub const DEFAULT_VERTEX_SHADER: &str = include_str!("shaders/unlit.vert");
/// The default fragment shader
pub const DEFAULT_FRAGMENT_SHADER: &str = include_str!("shaders/unlit.frag");

// repr(C) is for the host; makes uploading vertices efficient.
/// Vertex
#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Vertex {
    /// Local position
    pub pos: [f32; 3],
    /// Either u, v, w for textures or r, g, b for colors
    pub uvw: [f32; 3],
}

/// Unique identifier for a remote RenderData resource
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct MeshHandle(GenericHandle);
make_handle!(MeshHandle);

/// Unique identifier for a remote Shader program
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ShaderHandle(GenericHandle);
make_handle!(ShaderHandle);

/// Component denotes a camera
/// The Transform on the entity this is attached to will correspond to:
/// * VR: The position and orientation of the floor
/// * Desktop: The view matrix of the camera
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct CameraComponent {
    /// Background color
    pub clear_color: [f32; 3],
    /// Projection matrices
    /// * VR: Left and right eyes
    /// * Desktop: only the left eye is used
    pub projection: [Matrix4<f32>; 2],
}

/// All information required to define a renderable mesh
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadMesh {
    /// Mesh data
    pub mesh: Mesh,
    /// Unique ID
    pub id: MeshHandle,
}

/// A complete description of a shader (sources)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShaderSource {
    // TODO: Use SPIRV here? It's much more stable!
    /// Vertex shader source (GLSL)
    pub vertex_src: String,
    /// Fragment shader source (GLSL)
    pub fragment_src: String,
    /// Unique ID
    pub id: ShaderHandle,
}

/// Mesh defined by vertices and indices
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Mesh {
    /// Vertices. An empty list indicates procedurally generated vertex data
    pub vertices: Vec<Vertex>,
    /// Indices. An empty list indicates sequential vertex buffer usage
    pub indices: Vec<u32>,
}

/// Render component
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Render {
    /// Id of the associated RenderData
    pub id: MeshHandle,
    /// Primitive to construct this object
    pub primitive: Primitive,
    // /// * If no vertices, no indices: Vertex shader procedurally generates vertices
    // /// * If vertices, no indices: This many vertices are drawn
    // /// * If vertices, indices: This many indices are drawn
    // /// * If no vertices, indices: No object drawn. What are you trying to do??
    // /// * If limit == 0: Entire defined shape is drawn
    /// Use this many indices, in order
    /// Draw everything if None
    pub limit: Option<u32>,
    /// Optional shader handle; defaults to DEFAULT_VERTEX_SHADER, DEFAULT_FRAGMENT_SHADER
    pub shader: Option<ShaderHandle>,
}

/// Extra render data per component
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, PartialEq)]
pub struct RenderExtra(pub [f32; 4 * 4]);

/// How to draw the given mesh
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Primitive {
    Points,
    Lines,
    Triangles,
}

impl Default for Primitive {
    fn default() -> Self {
        Self::Triangles
    }
}

impl Component for Render {
    const ID: &'static str = pkg_namespace!("Render");
}

impl Component for RenderExtra {
    const ID: &'static str = pkg_namespace!("RenderExtra");
}

impl Message for UploadMesh {
    const CHANNEL: ChannelIdStatic = ChannelIdStatic {
        id: pkg_namespace!("RenderData"),
        locality: Locality::Local,
    };
}

impl Message for ShaderSource {
    const CHANNEL: ChannelIdStatic = ChannelIdStatic {
        id: pkg_namespace!("ShaderData"),
        locality: Locality::Local,
    };
}

impl Component for CameraComponent {
    const ID: &'static str = pkg_namespace!("CameraComponent");
}

impl Vertex {
    pub fn new(pos: [f32; 3], uvw: [f32; 3]) -> Self {
        Self { pos, uvw }
    }
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

impl Render {
    pub fn new(id: MeshHandle) -> Self {
        Self {
            id,
            primitive: Primitive::Triangles,
            shader: None,
            limit: None,
        }
    }

    pub fn primitive(mut self, primitive: Primitive) -> Self {
        self.primitive = primitive;
        self
    }

    pub fn shader(mut self, shader: ShaderHandle) -> Self {
        self.shader = Some(shader);
        self
    }

    pub fn limit(mut self, limit: Option<u32>) -> Self {
        self.limit = limit;
        self
    }
}

impl Mesh {
    /// Create a new mesh
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a Vertex and return it's index
    pub fn push_vertex(&mut self, v: Vertex) -> u32 {
        let idx: u32 = self
            .vertices
            .len()
            .try_into()
            .expect("Vertex limit exceeded");
        self.vertices.push(v);
        idx
    }

    /// Push an index
    pub fn push_indices(&mut self, idx: &[u32]) {
        self.indices.extend_from_slice(idx);
    }

    /// Erase all content
    pub fn clear(&mut self) {
        self.indices.clear();
        self.vertices.clear();
    }
}

#[cfg(test)]
mod tests {
    use cimvr_engine_interface::serial::serialized_size;

    use super::*;

    #[test]
    fn test_render_component() {
        let example = Render {
            id: MeshHandle::new(pkg_namespace!("Test render")),
            primitive: Primitive::Lines,
            limit: Some(90),
            shader: Some(ShaderHandle::new(pkg_namespace!("Test shader"))),
        };
        assert_eq!(
            serialized_size(&example).unwrap(),
            usize::from(Render::ID.size)
        );
    }
}

impl Default for CameraComponent {
    /// The default is an identity projection with a black clear color.
    fn default() -> Self {
        Self {
            clear_color: [0.; 3],
            projection: [Matrix4::identity(); 2],
        }
    }
}
