/*
extern crate glow as gl;
use anyhow::{bail, Context, Result};
use cimvr_engine::hotload::Hotloader;
use cimvr_engine::interface::prelude::{query, Access, Synchronized};
use cimvr_engine::interface::serial::deserialize;
use cimvr_engine::network::{
    length_delmit_message, AsyncBufferedReceiver, ClientToServer, ReadState, ServerToClient,
};
use cimvr_engine::Config;
use cimvr_engine::{interface::system::Stage, Engine};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use input::UserInputHandler;
use render::RenderPlugin;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;

mod input;
mod render;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ChatImproVR client",
    about = "Client application for experiencing the ChatImproVR metaverse"
)]
struct Opt {
    /// Remote host address, defaults to local server
    #[structopt(short, long, default_value = "127.0.0.1:5031")]
    connect: SocketAddr,

    /// Plugins
    plugins: Vec<PathBuf>,
}

struct Client {
    engine: Engine,
    render: RenderPlugin,
    input: UserInputHandler,
    recv_buf: AsyncBufferedReceiver,
    conn: TcpStream,
    hotload: Hotloader,
}

fn pmain() -> Result<()> {
    // Set up logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse args
    let args = Opt::from_args();

    // Connect to remote host
    let tcp_stream = TcpStream::connect(args.connect)?;
    tcp_stream.set_nonblocking(true)?;

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

    // Set up hotloading
    let hotload = Hotloader::new(&args.plugins)?;

    // Set up engine and initialize plugins
    let engine = Engine::new(&args.plugins, Config { is_server: false })?;

    // Setup client code
    let mut client = Client::new(engine, gl, tcp_stream, hotload)?;

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
                glutin_ctx.swap_buffers().unwrap();
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
    pub fn new(
        mut engine: Engine,
        gl: gl::Context,
        conn: TcpStream,
        hotload: Hotloader,
    ) -> Result<Self> {
        let render = RenderPlugin::new(gl, &mut engine).context("Setting up render engine")?;
        let input = UserInputHandler::new();

        // Initialize plugins AFTER we set up our plugins
        engine.init_plugins()?;

        Ok(Self {
            hotload,
            conn,
            recv_buf: AsyncBufferedReceiver::new(),
            engine,
            render,
            input,
        })
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.input.handle_winit_event(event);
        match event {
            WindowEvent::Resized(physical_size) => self.render.set_screen_size(*physical_size),
            _ => (),
        }
    }

    pub fn frame(&mut self) -> Result<()> {
        // Check for hotloaded plugins
        for path in self.hotload.hotload()? {
            log::info!("Reloading {}", path.display());
            self.engine.reload(path)?;
        }

        // Synchronize
        match self.recv_buf.read(&mut self.conn)? {
            ReadState::Invalid => {
                log::error!("Failed to parse invalid message");
            }
            ReadState::Incomplete => (),
            ReadState::Disconnected => {
                bail!("Disconnected");
            }
            ReadState::Complete(buf) => {
                // Update state!
                let recv: ServerToClient = deserialize(std::io::Cursor::new(buf))?;
                for msg in recv.messages {
                    self.engine.broadcast_local(msg);
                }
                self.engine
                    .ecs()
                    .import(&[query::<Synchronized>(Access::Read)], recv.ecs);
            }
        }

        // Pre-update
        self.engine.send(self.input.get_history());
        self.engine.dispatch(Stage::PreUpdate)?;

        // Update
        self.engine.dispatch(Stage::Update)?;

        // Post-update
        self.render.frame(&mut self.engine)?;
        self.engine.dispatch(Stage::PostUpdate)?;

        // Send message to server
        let msg = ClientToServer {
            messages: self.engine.network_inbox(),
        };
        length_delmit_message(&msg, &mut self.conn)?;

        Ok(())
    }
}
*/

//! Example how to use pure `egui_glow`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(unsafe_code)]

fn main() {
    let mut clear_color = [0.1, 0.1, 0.1];

    let event_loop = glutin::event_loop::EventLoopBuilder::with_user_event().build();
    let (gl_window, gl) = create_display(&event_loop);
    let gl = std::sync::Arc::new(gl);

    let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, gl.clone());

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let mut quit = false;

            let repaint_after = egui_glow.run(gl_window.window(), |egui_ctx| {
                egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
                    ui.heading("Hello World!");
                    if ui.button("Quit").clicked() {
                        quit = true;
                    }
                    ui.color_edit_button_rgb(&mut clear_color);
                });
            });

            *control_flow = if quit {
                glutin::event_loop::ControlFlow::Exit
            } else if repaint_after.is_zero() {
                gl_window.window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else if let Some(repaint_after_instant) =
                std::time::Instant::now().checked_add(repaint_after)
            {
                glutin::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                unsafe {
                    use glow::HasContext as _;
                    gl.clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }

                // draw things behind egui here

                egui_glow.paint(gl_window.window());

                // draw things on top of egui here

                gl_window.swap_buffers().unwrap();
            }
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            glutin::event::Event::WindowEvent { event, .. } => {
                use glutin::event::WindowEvent;
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                if let glutin::event::WindowEvent::Resized(physical_size) = &event {
                    gl_window.resize(*physical_size);
                } else if let glutin::event::WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    ..
                } = &event
                {
                    gl_window.resize(**new_inner_size);
                }

                egui_glow.on_event(&event);

                gl_window.window().request_redraw(); // TODO(emilk): ask egui if the events warrants a repaint instead
            }
            glutin::event::Event::LoopDestroyed => {
                egui_glow.destroy();
            }
            glutin::event::Event::NewEvents(glutin::event::StartCause::ResumeTimeReached {
                ..
            }) => {
                gl_window.window().request_redraw();
            }

            _ => (),
        }
    });
}

fn create_display(
    event_loop: &glutin::event_loop::EventLoop<()>,
) -> (
    glutin::WindowedContext<glutin::PossiblyCurrent>,
    glow::Context,
) {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("egui_glow example");

    let gl_window = unsafe {
        glutin::ContextBuilder::new()
            .with_depth_buffer(0)
            .with_srgb(true)
            .with_stencil_buffer(0)
            .with_vsync(true)
            .build_windowed(window_builder, event_loop)
            .unwrap()
            .make_current()
            .unwrap()
    };

    let gl = unsafe { glow::Context::from_loader_function(|s| gl_window.get_proc_address(s)) };

    (gl_window, gl)
}
