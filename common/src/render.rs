use cimvr_engine_interface::prelude::*;
use serde::{Deserialize, Serialize};

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

/// Unique handle to previously-sent RenderData
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Handle(pub u128);

/// All information required to define a renderable mesh
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RenderData {
    pub mesh: Mesh,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mesh {
    /// Vertices. An empty list indicates procedurally generated vertex data
    pub vertices: Vec<Vertex>,
    /// Indices. An empty list indicates sequential vertex buffer usage
    pub indices: Vec<u16>,
}

/// Render component
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Render {
    /// Id of the associated RenderData
    pub id: Handle,
    /// Primitive to construct this object
    pub primitive: Primitive,
    /// * If no vertices, no indices: Vertex shader procedurally generates vertices
    /// * If vertices, no indices: This many vertices are drawn
    /// * If vertices, indices: This many indices are drawn
    /// * If no vertices, indices: No object drawn. What are you trying to do??
    /// * If limit == 0: Entire defined shape is drawn
    pub limit: u32,
}

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
        size: 7,
    };
}

impl Message for RenderData {
    const CHANNEL: ChannelId = ChannelId {
        id: 0xCE0_0F_BEEF,
        locality: Locality::Local,
    };
}
