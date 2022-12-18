extern crate glow as gl;
use anyhow::{Context, Result};
use cimvr_common::{StringMessage, Transform};
use cimvr_engine::{
    interface::{
        prelude::{query, Access},
        system::Stage,
    },
    Engine,
};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use render::RenderEngine;
use std::path::PathBuf;

mod render;

struct Client {
    engine: Engine,
    render: RenderEngine,
}

fn main() -> Result<()> {
    // Parse args
    let args = std::env::args().skip(1);
    let paths: Vec<PathBuf> = args.map(PathBuf::from).collect();

    // Set up window
    let event_loop = glutin::event_loop::EventLoop::new();
    let window_builder = glutin::window::WindowBuilder::new().with_title("ChatImproVR");

    // Set up OpenGL
    let glutin_ctx = unsafe {
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(window_builder, &event_loop)?
            .make_current()
            .unwrap()
    };

    let gl = unsafe {
        gl::Context::from_loader_function(|s| glutin_ctx.get_proc_address(s) as *const _)
    };

    // Set up engine and initialize plugins
    let mut engine = Engine::new(&paths)?;
    engine.init_plugins()?;

    // Setup client code
    let mut client = Client::new(engine, gl)?;

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::LoopDestroyed => {
                return;
            }
            Event::MainEventsCleared => {
                glutin_ctx.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                client.frame().expect("Frame returned error");
            }
            Event::WindowEvent { ref event, .. } => {
                client.handle_event(event);
                match event {
                    WindowEvent::Resized(physical_size) => {
                        glutin_ctx.resize(*physical_size);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                }
            }
            _ => (),
        }
    });
}

impl Client {
    pub fn new(mut engine: Engine, gl: gl::Context) -> Result<Self> {
        let render = RenderEngine::new(gl, &mut engine).context("Setting up render engine")?;
        Ok(Self { engine, render })
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Resized(physical_size) => self.render.set_screen_size(*physical_size),
            _ => (),
        }
    }

    pub fn frame(&mut self) -> Result<()> {
        // Input stage
        self.engine.dispatch(Stage::Input)?;

        // Physics stage
        self.engine.dispatch(Stage::Physics)?;

        // Media stage
        self.render.frame(&mut self.engine);
        self.engine.dispatch(Stage::Media)?;

        Ok(())
    }
}
