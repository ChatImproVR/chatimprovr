extern crate glow as gl;
use anyhow::{bail, Context, Result};
use cimvr_engine::interface::prelude::{query, Access, Synchronized};
use cimvr_engine::interface::serial::deserialize;
use cimvr_engine::network::{
    length_delmit_message, AsyncBufferedReceiver, ClientToServer, ReadState, ServerToClient,
};
use cimvr_engine::{interface::system::Stage, Engine};
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use input::UserInputHandler;
use render::RenderPlugin;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;

mod input;
mod render;

use std::path::PathBuf;
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

    // Set up engine and initialize plugins
    let engine = Engine::new(&args.plugins, false)?;

    // Setup client code
    let mut client = Client::new(engine, gl, tcp_stream)?;

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
    pub fn new(mut engine: Engine, gl: gl::Context, conn: TcpStream) -> Result<Self> {
        let render = RenderPlugin::new(gl, &mut engine).context("Setting up render engine")?;
        let input = UserInputHandler::new();

        // Initialize plugins AFTER we set up our plugins
        engine.init_plugins()?;

        Ok(Self {
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
                    self.engine.broadcast(msg);
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
        length_delmit_message(&msg, self.conn)?;

        Ok(())
    }
}
