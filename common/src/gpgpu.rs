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

#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct GpuBuffer {
    pub id: GenericHandle,
    pub typ: BufferType,
    pub layout: BufferLayout,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum BufferType {
    Storage,
    Uniform,
}

fn upload_buffer(io: &mut EngineIo, typ: BufferType, layout: BufferLayout, n: usize, bytes: &[u8]) {
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum BufferLayout {
    Float,
    Vec2,
    //No vec3, because it's inefficient on most hardware! Just use vec4.
    Vec4,
    Vertex,
    //Int,
    //IVec2,
    //Ivec4,
    //Uint,
    //UVec2,
    //UVec4,
    //Custom(CustomBufferLayout) ( Soon™ )
}

/// A buffer and it's associated data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BufferPacket {
    pub id: GpuBuffer,
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
#[derive(Serialize, Deserialize, Copy, Debug, Clone)]
enum ComputeOperation {
    InvokeComputeShader(ComputeShader, i32, i32, i32),
    CopyToVertices(GpuBuffer, MeshHandle),
    CopyToIndices(GpuBuffer, MeshHandle),
}

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[locality("Local")]
pub struct GpuDeclarePipeline {
    stages: Vec<ComputeShader>,
}
