//! Types for interfacing with the Host's rendering engine
use bytemuck::{Pod, Zeroable};
use cimvr_engine_interface::{pkg_namespace, prelude::*, serial::FixedOption};
use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::{make_handle, render::MeshHandle, GenericHandle};

// const COMPUTE_

/// Unique identifier for a Compute Shader
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ComputeShader(GenericHandle);
make_handle!(ComputeShader);

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct GpuBufferHandle {
    pub handle: GenericHandle,
    pub typ: BufferType,
    pub layout: BufferLayout,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum BufferType {
    Storage,
    Uniform,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum BufferLayout {
    Float(usize),
    Vec2(usize),
    //No vec3, because it's inefficient on most hardware! Just use vec4.
    Vec4(usize),
    /// RGBA 2D floating point image with dimensions of (width, height)
    // /// Note that each image is ALSO supplied as a sample-able texture!
    Image2D(usize, usize),
    //Image3D,
    //Vertex,
    //Int,
    //IVec2,
    //Ivec4,
    //Uint,
    //UVec2,
    //UVec4,
    //Custom(CustomBufferLayout) ( Soon™ )
}

impl BufferLayout {
    pub fn size(&self) -> usize {
        match self {
            BufferLayout::Float(n) => 4 * n,
            BufferLayout::Vec2(n) => 4 * 2 * n,
            BufferLayout::Vec4(n) => 4 * 4 * n,
            BufferLayout::Image2D(w, h) => 4 * 4 * w * h,
        }
    }
}

/// A buffer and it's associated data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BufferPacket {
    pub handle: GpuBufferHandle,
    pub data: Vec<u8>,
}

/// plugin to GPU data transfer
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[locality("Local")]
pub struct GpuBufferUpload(BufferPacket);

/// GPU to plugin data transfer
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[locality("Local")]
pub struct GpuBufferDownload(BufferPacket);

/// GPGPU compute operation
#[derive(Serialize, Deserialize, Debug, Clone)]
enum GpuComputeOperation {
    InvokeComputeShader {
        shader: ComputeShader,
        x: i32,
        y: i32,
        z: i32,
        buffers: Vec<GpuBufferHandle>,
    },
    /// Executes a builtin shader which copies consecutive pairs of vec4s from the source buffer
    /// into the color and uv fields of the vertices of the referenced mesh
    /// So, if you have a mesh with N vertices, N*2 Vec4s.
    CopyToVertices(GpuBufferHandle, MeshHandle),
    //CopyToIndices(GpuBuffer, MeshHandle),
}

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[locality("Local")]
pub struct GpuDeclarePipeline {
    stages: Vec<ComputeShader>,
}

impl BufferPacket {
    pub fn new(handle: GpuBufferHandle, data: Vec<u8>) -> Self {
        Self { handle, data }
    }
}

impl GpuBufferHandle {
    pub const fn new(name: &str, typ: BufferType, layout: BufferLayout) -> Self {
        let handle = GenericHandle::new(name);
        Self {
            handle,
            typ,
            layout,
        }
    }

    pub fn index(self, i: u128) -> Self {
        let mut out = self;
        out.handle = out.handle.index(i);
        out
    }
}
