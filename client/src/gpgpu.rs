use std::collections::HashMap;

use cimvr_common::gpgpu::{ComputeShaderHandle, GpuShaderUpload, ComputeJob, ComputeOperation};
use anyhow::Result;
use gl::HasContext;

use crate::render::compile_glsl_program;

pub struct ComputeEngine {
    shaders: HashMap<ComputeShaderHandle, GpuComputeShader>,
}

impl ComputeEngine {
    pub fn new(gl: &gl::Context) -> Self {
        Self {
            shaders: Default::default()
        }
    }

    pub fn upload_shader(&mut self, gl: &gl::Context, upload: GpuShaderUpload) -> Result<()> {
        let GpuShaderUpload(handle, code) = upload;
        let shader = GpuComputeShader::new(gl, &code)?;
        self.shaders.insert(handle, shader);
        Ok(())
    }

    pub fn run_compute_job(job: ComputeJob, rdr: RenderEngine) -> Result<()> {
        for step in job {
            match step {
                ComputeOperation::InvokeComputeShader { shader, x, y, z, buffers, uniforms } => {
                },
                ComputeOperation::CopyToVertices(buffer, mesh) => {
                    todo!(),
                }
            }
        }

        Ok(())
    }
}

// TODO: Add deallocation!
struct GpuComputeShader {
    program: gl::Program,
}

impl GpuComputeShader {
    fn new(gl: &gl::Context, src: &str) -> Result<Self> {
        let program = compile_glsl_program(gl, &[(gl::COMPUTE_SHADER, src)])?;
        Ok(Self { program })
    }
}
