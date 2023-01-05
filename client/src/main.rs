extern crate glow as gl;
use anyhow::{bail, Context, Result};
use cimvr_engine::hotload::Hotloader;
use cimvr_engine::interface::prelude::{Access, QueryComponent, Synchronized};
use cimvr_engine::interface::serial::deserialize;
use cimvr_engine::network::{
    length_delmit_message, AsyncBufferedReceiver, ClientToServer, ReadState, ServerToClient,
};
use cimvr_engine::Config;
use cimvr_engine::{interface::system::Stage, Engine};
use egui_glow::EguiGlow;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use input::UserInputHandler;
use render::RenderPlugin;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use ui::OverlayUi;

mod input;
mod render;
mod ui;

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
    egui_glow: EguiGlow,
    ui: OverlayUi,
}

fn main() -> Result<()> {
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
    let gl = std::sync::Arc::new(gl);

    // Set up hotloading
    let hotload = Hotloader::new(&args.plugins)?;

    // Set up engine and initialize plugins
    let engine = Engine::new(&args.plugins, Config { is_server: false })?;

    // Set up egui
    let egui_glow = egui_glow::EguiGlow::new(&event_loop, gl.clone());

    // Setup client code
    let mut client = Client::new(engine, gl, tcp_stream, hotload, egui_glow)?;

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        client.handle_event(&event);
        match event {
            Event::LoopDestroyed => {
                return;
            }
            Event::MainEventsCleared => {
                glutin_ctx.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                client
                    .frame(glutin_ctx.window())
                    .expect("Frame returned error");
                glutin_ctx.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    glutin_ctx.resize(*physical_size);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            _ => (),
        }
    });
}

impl Client {
    pub fn new(
        mut engine: Engine,
        gl: std::sync::Arc<gl::Context>,
        conn: TcpStream,
        hotload: Hotloader,
        egui_glow: EguiGlow,
    ) -> Result<Self> {
        let render = RenderPlugin::new(gl, &mut engine).context("Setting up render engine")?;
        let input = UserInputHandler::new();
        let ui = OverlayUi::new(&mut engine);

        // Initialize plugins AFTER we set up our plugins
        engine.init_plugins()?;

        Ok(Self {
            egui_glow,
            hotload,
            conn,
            ui,
            recv_buf: AsyncBufferedReceiver::new(),
            engine,
            render,
            input,
        })
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { event, .. } => {
                if !self.egui_glow.on_event(&event) {
                    self.input.handle_winit_event(event);
                }
                match event {
                    WindowEvent::Resized(physical_size) => {
                        self.render.set_screen_size(*physical_size)
                    }
                    _ => (),
                }
            }
            Event::LoopDestroyed => {
                self.egui_glow.destroy();
            }
            _ => (),
        }
    }

    pub fn frame(&mut self, window: &glutin::window::Window) -> Result<()> {
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
                self.engine.ecs().import(
                    &[QueryComponent::new::<Synchronized>(Access::Read)],
                    recv.ecs,
                );
            }
        }

        // Pre-update
        self.engine.send(self.input.get_history());
        self.engine.dispatch(Stage::PreUpdate)?;

        // Update
        self.engine.dispatch(Stage::Update)?;

        // UI updates
        self.ui.update(&mut self.engine);
        self.egui_glow
            .run(window, |ctx| self.ui.run(ctx, &mut self.engine));

        // Render game, then egui
        self.render.frame(&mut self.engine)?;
        self.egui_glow.paint(window);

        // Post-update
        self.engine.dispatch(Stage::PostUpdate)?;

        // Send message to server
        let msg = ClientToServer {
            messages: self.engine.network_inbox(),
        };
        length_delmit_message(&msg, &mut self.conn)?;
        self.conn.flush()?;

        Ok(())
    }
}

/*
fn pmain() {
    let mut clear_color = [0.1, 0.1, 0.1];

    let event_loop = glutin::event_loop::EventLoopBuilder::with_user_event().build();
    let (gl_window, gl) = create_display(&event_loop);

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let mut quit = false;

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
*/
