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
        loop {
            match self.recv_buf.read(&mut self.conn)? {
                ReadState::Invalid => {
                    log::error!("Failed to parse invalid message");
                }
                ReadState::Incomplete => break,
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
