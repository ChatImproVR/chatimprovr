use anyhow::bail;
use anyhow::format_err;
use anyhow::Result;
use cimvr_common::glam::Mat4;
use cimvr_common::{render::*, Transform};
use cimvr_engine::interface::prelude::*;
use cimvr_engine::{interface::{pkg_namespace, component_id}, Engine};
use gl::HasContext;
use glow::NativeUniformLocation;

use std::collections::HashMap;
use std::sync::Arc;

const DEFAULT_SHADER: ShaderHandle = ShaderHandle::new(pkg_namespace!("Default shader"));

/// Rendering Plugin, containing interfacing with ChatImproVR for RenderEngine
pub struct RenderPlugin {
    gl: Arc<glow::Context>,
    rdr: RenderEngine,
}

// TODO: destructors! (lol)
/// Rendering engine state
struct RenderEngine {
    meshes: HashMap<MeshHandle, GpuMesh>,
    shaders: HashMap<ShaderHandle, GpuShader>,
}

// TODO: Actual mesh memory management. Fewer buffers!!
/// Mesh data on GPU
struct GpuMesh {
    /// Vertex array
    vao: gl::VertexArray,
    /// Vertex buffer (vertices)
    vbo: gl::NativeBuffer,
    /// Element buffer (indices)
    ebo: gl::NativeBuffer,
    /// Number of indices in this mesh
    index_count: i32,
}

struct GpuShader {
    program: gl::Program,
    transf_loc: Option<NativeUniformLocation>,
    extra_loc: Option<NativeUniformLocation>,
}

impl RenderPlugin {
    pub fn new(gl: Arc<gl::Context>, engine: &mut Engine) -> Result<Self> {
        engine.subscribe::<UploadMesh>();
        engine.subscribe::<ShaderSource>();

        let rdr = RenderEngine::new(&gl)?;

        Ok(Self { gl, rdr })
    }

    pub fn set_screen_size(&mut self, width: u32, height: u32) {
        unsafe {
            self.gl.scissor(0, 0, width as i32, height as i32);
            self.gl.viewport(0, 0, width as i32, height as i32);
        }
    }

    /// Draw a frame, prepending camera transform to the given view
    pub fn frame(
        &mut self,
        engine: &mut Engine,
        vr_view: Mat4,
        camera_idx: usize,
    ) -> Result<()> {
        // Upload render data
        for msg in engine.inbox::<UploadMesh>() {
            if let Err(e) = self.rdr.upload_render_data(&self.gl, &msg) {
                log::error!("Error uploading render data at id {:?}; {:?}", msg.id, e);
            }
        }

        // Upload shader
        for msg in engine.inbox::<ShaderSource>() {
            if let Err(e) = self.rdr.upload_shader(&self.gl, &msg) {
                log::error!("Error uploading shader at id {:?}; {:?}", msg.id, e);
            }
        }

        // Find camera, if any
        let camera_entity = match engine
            .ecs()
            .find(&[component_id::<CameraComponent>(), component_id::<Transform>()])
        {
            Some(c) => c,
            None => {
                log::warn!("No Camera found! Did you attach both Transform and CameraComponent?");
                return Ok(());
            }
        };

        let camera_transf = engine.ecs().get::<Transform>(camera_entity).unwrap();
        let camera_comp = engine.ecs().get::<CameraComponent>(camera_entity).unwrap();
        let proj = camera_comp.projection[camera_idx];
        let view = vr_view * camera_transf.view();

        // Draw!
        self.rdr.start_frame(&self.gl, camera_comp.clear_color)?;

        // Prepare data
        let entities = engine.ecs().query(&[
            QueryComponent::new::<Render>(Access::Read),
            QueryComponent::new::<Transform>(Access::Read),
        ]);

        for entity in entities {
            let transf = engine.ecs().get::<Transform>(entity).unwrap();
            let rdr_comp = engine.ecs().get::<Render>(entity).unwrap();

            // TODO: Sort entities by shader in order to set this less!
            let wanted_shader: Option<_> = rdr_comp.shader.into();
            let wanted_shader = wanted_shader.unwrap_or(DEFAULT_SHADER);
            let extra = engine.ecs().get::<RenderExtra>(entity);

            let res = self
                .rdr
                .set_shader(&self.gl, wanted_shader, view, proj, transf, extra);

            if let Err(e) = res {
                log::error!("Error setting shader for entity {:?}; {:?}", entity, e);
                continue;
            }

            if let Err(e) = self.rdr.draw(&self.gl, rdr_comp) {
                log::error!(
                    "Error drawing render component {:?} on entity {:?}; {:?}",
                    rdr_comp,
                    entity,
                    e
                );
                continue;
            }
        }

        Ok(())
    }
}

impl RenderEngine {
    pub fn new(gl: &gl::Context) -> Result<Self> {
        unsafe {
            // Enable backface culling
            gl.enable(gl::CULL_FACE);

            // Enable depth buffering
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS);

            // Enable point size to be determined in shaders
            gl.enable(gl::VERTEX_PROGRAM_POINT_SIZE);

            // Compile shaders
            let mut shaders = HashMap::new();
            shaders.insert(
                DEFAULT_SHADER,
                GpuShader::new(gl, DEFAULT_FRAGMENT_SHADER, DEFAULT_VERTEX_SHADER)?,
            );

            Ok(Self {
                meshes: HashMap::new(),
                shaders,
            })
        }
    }

    /// Upload shader data
    pub fn upload_shader(&mut self, gl: &gl::Context, data: &ShaderSource) -> Result<()> {
        // TODO: Unload old shader
        let shader = GpuShader::new(gl, &data.fragment_src, &data.vertex_src)?;
        self.shaders.insert(data.id, shader);
        Ok(())
    }

    /// Make the given render data available to the GPU
    pub fn upload_render_data(&mut self, gl: &gl::Context, data: &UploadMesh) -> Result<()> {
        // TODO: Use a different mesh type? Switch for upload frequency? Hmmm..
        if let Some(buf) = self.meshes.get_mut(&data.id) {
            update_mesh(gl, buf, &data.mesh);
        } else {
            let gpu_mesh =
                upload_mesh(gl, gl::DYNAMIC_DRAW, &data.mesh).expect("Failed to upload mesh");
            self.meshes.insert(data.id, gpu_mesh);
        }

        Ok(())
    }

    /// Begin a new frame (clears buffer, sets uniforms)
    pub fn start_frame(&mut self, gl: &gl::Context, clear_color: [f32; 3]) -> Result<()> {
        unsafe {
            // Clear depth and color buffers
            gl.disable(gl::BLEND);
            gl.disable(gl::SCISSOR_TEST);
            gl.disable(gl::STENCIL_TEST);
            gl.enable(gl::FRAMEBUFFER_SRGB);

            gl.enable(gl::CULL_FACE);
            gl.enable(glow::DEPTH_TEST);

            let [r, g, b] = clear_color;
            gl.clear_color(r, g, b, 1.0);
            gl.depth_func(glow::LESS);
            gl.depth_mask(true);
            gl.depth_range_f32(0., 1.);
            gl.clear_depth_f32(1.0);

            gl.clear(gl::COLOR_BUFFER_BIT | gl::STENCIL_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl.enable(gl::BLEND);
            gl.blend_func(gl::ONE, gl::ZERO);

        }

        Ok(())
    }

    /// Set the current shader
    pub fn set_shader(
        &mut self,
        gl: &gl::Context,
        handle: ShaderHandle,
        view: Mat4,
        proj: Mat4,
        transf: Transform,
        extra: Option<RenderExtra>,
    ) -> Result<()> {
        if let Some(shader) = self.shaders.get(&handle) {
            shader.load(gl, view, proj);
            shader.set_uniforms(gl, transf, extra);
            Ok(())
        } else {
            log::trace!("Shader handle {:?} not found", handle);
            Ok(())
        }
    }

    /// Draw the specified render component
    pub fn draw(&mut self, gl: &gl::Context, rdr_comp: Render) -> Result<()> {
        if let Some(mesh) = self.meshes.get(&rdr_comp.id) {
            mesh.draw(gl, rdr_comp)?;
        } else {
            bail!("Attempted to access absent mesh data {:?}", rdr_comp.id);
        }

        Ok(())
    }
}

impl GpuShader {
    fn new(gl: &gl::Context, fragment: &str, vertex: &str) -> Result<Self> {
        let sources = [(gl::VERTEX_SHADER, vertex), (gl::FRAGMENT_SHADER, fragment)];

        let program = compile_glsl_program(gl, &sources)?;
        let (transf_loc, extra_loc) = unsafe {
            (
                gl.get_uniform_location(program, "transf"),
                gl.get_uniform_location(program, "extra"),
            )
        };

        Ok(Self {
            program,
            transf_loc,
            extra_loc,
        })
    }

    fn load(&self, gl: &gl::Context, view: Mat4, proj: Mat4) {
        unsafe {
            // Draw map
            // Must bind program before setting uniforms!!!
            gl.use_program(Some(self.program));

            // Set camera matrix
            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(self.program, "view").as_ref(),
                false,
                &view.to_cols_array()
            );

            gl.uniform_matrix_4_f32_slice(
                gl.get_uniform_location(self.program, "proj").as_ref(),
                false,
                &proj.to_cols_array()
            );
        }
    }

    fn set_uniforms(&self, gl: &gl::Context, transf: Transform, extra: Option<RenderExtra>) {
        // Set transform
        let matrix = transf.to_homogeneous();
        unsafe {
            gl.uniform_matrix_4_f32_slice(
                self.transf_loc.as_ref(),
                false,
                bytemuck::cast_slice(matrix.as_ref()),
            );
        }

        if let Some(RenderExtra(data)) = extra {
            unsafe {
                gl.uniform_matrix_4_f32_slice(
                    self.extra_loc.as_ref(),
                    false,
                    bytemuck::cast_slice(data.as_ref()),
                );
            }
        }
    }
}

/// Compiles (*_SHADER, <source>) into a shader program for OpenGL
fn compile_glsl_program(gl: &gl::Context, sources: &[(u32, &str)]) -> Result<gl::Program> {
    // Compile default shaders
    unsafe {
        let program = gl.create_program().expect("Cannot create program");

        let mut shaders = vec![];

        for (stage, shader_source) in sources {
            let shader = gl.create_shader(*stage).expect("Cannot create shader");

            gl.shader_source(shader, shader_source);

            gl.compile_shader(shader);

            if !gl.get_shader_compile_status(shader) {
                return Err(format_err!(
                    "OpenGL compile shader: {}",
                    gl.get_shader_info_log(shader)
                ));
            }

            gl.attach_shader(program, shader);

            shaders.push(shader);
        }

        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            return Err(format_err!(
                "OpenGL link shader: {}",
                gl.get_program_info_log(program)
            ));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        Ok(program)
    }
}

/// Set the vertex attribute corresponding to Vertex
fn set_vertex_attrib(gl: &gl::Context) {
    unsafe {
        // Set vertex attributes
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(
            0,
            3,
            gl::FLOAT,
            false,
            std::mem::size_of::<Vertex>() as i32,
            0,
        );

        gl.enable_vertex_attrib_array(1);
        gl.vertex_attrib_pointer_f32(
            1,
            3,
            gl::FLOAT,
            false,
            std::mem::size_of::<Vertex>() as i32,
            3 * std::mem::size_of::<f32>() as i32,
        );
    }
}

/// Uploads a mesh; does not unbind vertex array
fn upload_mesh(gl: &gl::Context, usage: u32, mesh: &Mesh) -> Result<GpuMesh, String> {
    unsafe {
        // Map buffer
        let vao = gl.create_vertex_array()?;
        let vbo = gl.create_buffer()?;
        let ebo = gl.create_buffer()?;

        gl.bind_vertex_array(Some(vao));

        // Write vertices
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(
            gl::ARRAY_BUFFER,
            bytemuck::cast_slice(&mesh.vertices),
            usage,
        );

        // Write vertices
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(
            gl::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&mesh.indices),
            usage,
        );

        // Set vertex attributes
        set_vertex_attrib(gl);

        // Unbind vertex array
        gl.bind_vertex_array(None);

        Ok(GpuMesh {
            vao,
            vbo,
            ebo,
            index_count: mesh.indices.len() as i32,
        })
    }
}

/// Upload mesh data to the GPU
fn update_mesh(gl: &gl::Context, buf: &mut GpuMesh, mesh: &Mesh) {
    unsafe {
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(buf.vbo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&mesh.vertices),
            glow::DYNAMIC_DRAW,
        );

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(buf.ebo));
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&mesh.indices),
            glow::DYNAMIC_DRAW,
        );

        buf.index_count = mesh.indices.len() as i32;
    }
}

impl GpuMesh {
    fn draw(&self, gl: &gl::Context, rdr_comp: Render) -> Result<()> {
        // Translate draw call
        let primitive = match rdr_comp.primitive {
            Primitive::Lines => gl::LINES,
            Primitive::Points => gl::POINTS,
            Primitive::Triangles => gl::TRIANGLES,
        };

        let limit: Option<u32> = rdr_comp.limit.into();
        let limit: i32 = match limit {
            None => self.index_count,
            Some(lim) => lim.try_into().unwrap(),
        };

        // Draw mesh data
        if limit <= self.index_count {
            unsafe {
                gl.bind_vertex_array(Some(self.vao));
                gl.draw_elements(primitive, limit, gl::UNSIGNED_INT, 0);
                Ok(())
            }
        } else {
            bail!(
                "Invalid draw limit, got {} but mesh has {} indices",
                limit,
                self.index_count
            );
        }
        //gl.bind_vertex_array(None);
    }
}
