use anyhow::Result;
use cimvr_common::{render::*, Transform};
use cimvr_engine::{
    interface::prelude::{query, Access},
    Engine,
};
use gl::HasContext;
use glutin::dpi::PhysicalSize;

pub struct RenderEngine {
    gl: gl::Context,
}

impl RenderEngine {
    pub fn new(gl: gl::Context, engine: &mut Engine) -> Result<Self> {
        engine.subscribe::<RenderData>();

        Ok(Self { gl })
    }

    pub fn set_screen_size(&mut self, size: PhysicalSize<u32>) {
        unsafe {
            self.gl.scissor(0, 0, size.width as i32, size.height as i32);
            self.gl
                .viewport(0, 0, size.width as i32, size.height as i32);
        }
    }

    pub fn frame(&mut self, engine: &mut Engine) {
        /*
        for msg in engine.inbox::<RenderData>() {
            dbg!(msg);
        }

        let entities = engine.ecs().query(&[
            query::<Render>(Access::Read),
            query::<Transform>(Access::Read),
        ]);

        for entity in entities {
            dbg!(engine.ecs().get::<Render>(entity));
            dbg!(engine.ecs().get::<Transform>(entity));
        }
        */
    }
}
