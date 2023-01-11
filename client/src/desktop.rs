use anyhow::{bail, format_err, Context, Result};
use cimvr_common::vr::{VrFov, VrUpdate};
use cimvr_common::Transform;
use cimvr_engine::hotload::Hotloader;
use cimvr_engine::interface::prelude::{Access, QueryComponent, Synchronized};
use cimvr_engine::interface::serial::deserialize;
use cimvr_engine::network::{
    length_delmit_message, AsyncBufferedReceiver, ClientToServer, ReadState, ServerToClient,
};
use cimvr_engine::Config;
use cimvr_engine::{interface::system::Stage, Engine};
use crate::desktop_input::DesktopInputHandler;
use egui_glow::EguiGlow;
use gl::HasContext;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use nalgebra::{Matrix4, Point3, Quaternion, Unit};
use render::RenderPlugin;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use ui::OverlayUi;
use crate::{render, ui};

use crate::{Opt, Client};

pub fn desktop(args: Opt) -> Result<()> {
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
    let gl = Arc::new(gl);

    // Set up egui
    let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, gl.clone());

    // Set up desktop input
    let mut input = DesktopInputHandler::new();

    // Setup client code
    let mut client = Client::new(gl, &args.plugins, args.connect)?;

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => {
                glutin_ctx.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                // Download messages from server
                client.download().expect("Message download");

                // Send input history
                client.engine().send(input.get_history());

                // Pre update stage
                client
                    .engine()
                    .dispatch(Stage::PreUpdate)
                    .expect("Frame pre-update");

                // Update stage
                client
                    .engine()
                    .dispatch(Stage::Update)
                    .expect("Frame udpate");

                // Collect UI input
                egui_glow.run(glutin_ctx.window(), |ctx| client.update_ui(ctx));

                // Render frame
                client
                    .render_frame(Matrix4::identity(), 0)
                    .expect("Frame render");

                // Render UI
                egui_glow.paint(glutin_ctx.window());

                // Post update stage
                client
                    .engine()
                    .dispatch(Stage::PostUpdate)
                    .expect("Frame post-update");

                // Upload messages to server
                client.upload().expect("Message upload");

                glutin_ctx.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => {
                if !egui_glow.on_event(&event) {
                    input.handle_winit_event(event);
                }

                match event {
                    WindowEvent::Resized(ph) => {
                        client.set_resolution(ph.width, ph.height);
                        glutin_ctx.resize(*ph);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    _ => (),
                }
            }
            Event::LoopDestroyed => {
                egui_glow.destroy();
            }
            _ => (),
        }
    });
}
