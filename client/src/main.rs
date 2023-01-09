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
use desktop::DesktopInputHandler;
use egui_glow::EguiGlow;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use render::RenderPlugin;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use ui::OverlayUi;

mod desktop;
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

    /// Whether to use VR
    #[structopt(long)]
    vr: bool,
}

struct Client {
    engine: Engine,
    render: RenderPlugin,
    recv_buf: AsyncBufferedReceiver,
    conn: TcpStream,
    hotload: Hotloader,
    ui: OverlayUi,
}

fn main() -> Result<()> {
    // Set up logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse args
    let args = Opt::from_args();

    if args.vr {
        virtual_reality(args)
    } else {
        desktop(args)
    }
}

fn desktop(args: Opt) -> Result<()> {
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

    // Set up egui
    let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, gl.clone());

    // Set up desktop input
    let mut input = DesktopInputHandler::new();

    // Setup client code
    let mut client = Client::new(gl, &args.plugins, args.connect)?;

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        client.handle_event(&event);
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
                client.render_frame().expect("Frame render");

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
                    WindowEvent::Resized(physical_size) => {
                        glutin_ctx.resize(*physical_size);
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

fn virtual_reality(args: Opt) -> Result<()> {
    todo!()
}

impl Client {
    pub fn new(
        gl: std::sync::Arc<gl::Context>,
        plugins: &[PathBuf],
        addr: SocketAddr,
    ) -> Result<Self> {
        // Connect to remote host
        let conn = TcpStream::connect(addr)?;
        conn.set_nonblocking(true)?;

        // Set up hotloading
        let hotload = Hotloader::new(&plugins)?;

        // Set up engine and initialize plugins
        let mut engine = Engine::new(&plugins, Config { is_server: false })?;

        // Set up rendering
        let render = RenderPlugin::new(gl, &mut engine).context("Setting up render engine")?;

        let ui = OverlayUi::new(&mut engine);

        // Initialize plugins AFTER we set up our plugins
        engine.init_plugins()?;

        Ok(Self {
            hotload,
            conn,
            ui,
            recv_buf: AsyncBufferedReceiver::new(),
            engine,
            render,
        })
    }

    pub fn handle_event(&mut self, event: &Event<()>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => self.render.set_screen_size(*physical_size),
                _ => (),
            },
            _ => (),
        }
    }

    /// Synchronize with remote and with plugin hotloading
    pub fn download(&mut self) -> Result<()> {
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
                ReadState::Incomplete => break Ok(()),
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
    }

    pub fn update_ui(&mut self, ctx: &egui::Context) {
        self.ui.update(&mut self.engine);
        self.ui.run(ctx, &mut self.engine);
    }

    pub fn render_frame(&mut self) -> Result<()> {
        self.render.frame(&mut self.engine)
    }

    pub fn upload(&mut self) -> Result<()> {
        // Send message to server
        let msg = ClientToServer {
            messages: self.engine.network_inbox(),
        };
        length_delmit_message(&msg, &mut self.conn)?;
        self.conn.flush()?;

        Ok(())
    }

    fn engine(&mut self) -> &mut Engine {
        &mut self.engine
    }
}
