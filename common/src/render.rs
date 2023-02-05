//! Types for interfacing with the Host's rendering engine
use bytemuck::{Pod, Zeroable};
use cimvr_engine_interface::prelude::*;
use nalgebra::Matrix4;
use serde::{Deserialize, Serialize};

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
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct RenderHandle(pub u128);

/// Unique identifier for a remote Shader program
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ShaderHandle(pub u128);

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
pub struct RenderData {
    /// Mesh data
    pub mesh: Mesh,
    /// Unique ID
    pub id: RenderHandle,
}

/// A complete description of a shader (sources)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShaderData {
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
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Render {
    /// Id of the associated RenderData
    pub id: RenderHandle,
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
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct RenderExtra(pub [f32; 4 * 4]);

/// How to draw the given mesh
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Primitive {
    Points,
    Lines,
    Triangles,
}

/// Information about the display; may be a window or a VR headset
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Screen {
    pub width: u32,
    pub height: u32,
}

impl Message for Screen {
    const CHANNEL: ChannelId = ChannelId {
        id: 0x234980,
        locality: Locality::Local,
    };
}

impl Component for Render {
    const ID: ComponentId = ComponentId {
        id: 0xDD05,
        size: 42,
    };
}

impl Component for RenderExtra {
    const ID: ComponentId = ComponentId {
        id: 0x328409D,
        size: 4 * 4 * 4,
    };
}

impl Message for RenderData {
    const CHANNEL: ChannelId = ChannelId {
        id: 0xCE0_0F_BEEF,
        locality: Locality::Local,
    };
}

impl Message for ShaderData {
    const CHANNEL: ChannelId = ChannelId {
        id: 0xBAD_BAE,
        locality: Locality::Local,
    };
}

impl Component for CameraComponent {
    const ID: ComponentId = ComponentId {
        id: 0x1337_1337_1337_1337_1337_1337_1337_1337,
        size: 156,
    };
}

impl Vertex {
    pub fn new(pos: [f32; 3], uvw: [f32; 3]) -> Self {
        Self { pos, uvw }
    }
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

impl Render {
    pub fn new(id: RenderHandle) -> Self {
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
            id: RenderHandle(23910),
            primitive: Primitive::Lines,
            limit: Some(90),
            shader: Some(ShaderHandle(93420)),
        };
        assert_eq!(
            serialized_size(&example).unwrap(),
            usize::from(Render::ID.size)
        );
    }
}
