use anyhow::Result;
use cimvr_common::{render::*, Transform};
use gl::HasContext;
use glutin::dpi::PhysicalSize;

pub struct RenderEngine {
    gl: gl::Context,
}

impl RenderEngine {
    pub fn new(gl: gl::Context) -> Result<Self> {
        Ok(Self { gl })
    }

    pub fn set_screen_size(&mut self, size: PhysicalSize<u32>) {
        unsafe {
            self.gl.scissor(0, 0, size.width as i32, size.height as i32);
            self.gl
                .viewport(0, 0, size.width as i32, size.height as i32);
        }
    }
}
