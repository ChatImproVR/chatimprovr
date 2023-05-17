extern crate glow as gl;

#[cfg(feature = "vr")]
extern crate openxr as xr;

use anyhow::{bail, Context, Result};
use cimvr_common::glam::Mat4;
use cimvr_engine::hotload::Hotloader;
use cimvr_engine::interface::prelude::{
    Access, ConnectionRequest, ConnectionResponse, PluginData, Query, Synchronized,
};
use cimvr_engine::interface::serial::{deserialize, serialize};
use cimvr_engine::network::{
    length_delimit_message, AsyncBufferedReceiver, ClientToServer, ReadState, ServerToClient,
};
use cimvr_engine::{Config, calculate_digest};
use cimvr_engine::Engine;
use directories::ProjectDirs;
use gamepad::GamepadPlugin;
use plugin_cache::FileCache;
use render::RenderPlugin;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use ui::OverlayUi;

#[cfg(feature = "vr")]
mod vr;

mod desktop;
mod desktop_input;
mod gamepad;
mod plugin_cache;
mod render;
mod ui;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ChatImproVR client",
    about = "Client application for experiencing the ChatImproVR metaverse"
)]
pub struct Opt {
    /// Remote host address, defaults to local server
    #[structopt(short, long)]
    pub connect: Option<String>,

    /// Whether to use VR
    #[structopt(long)]
    pub vr: bool,

    /// Username (optional, defaults to anonymousXXXX)
    #[structopt(short, long)]
    pub username: Option<String>,
}

struct Client {
    engine: Engine,
    render: RenderPlugin,
    recv_buf: AsyncBufferedReceiver,
    conn: TcpStream,
    gamepad: GamepadPlugin,
    ui: OverlayUi,
}

fn main() -> Result<()> {
    // Set up logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse args
    let mut args = Opt::from_args();
    let anonymous_user = format!("anon{:04}", random_number() % 10_000);
    args.username = args.username.or(Some(anonymous_user));

    if args.vr {
        #[cfg(not(feature = "vr"))]
        bail!("Client was not compiled with the \"vr\" feature. Virtual Reality is not available.");

        #[cfg(feature = "vr")]
        vr::mainloop(args)
    } else {
        desktop::mainloop(args)
    }
}

// TODO: Make it easier to add more plugins to both VR and Desktop, without introducing any more
// code uplication!

impl Client {
    pub fn new(gl: Arc<gl::Context>, addr: String, username: String) -> Result<Self> {
        // Set up plugin cache
        let mut plugin_cache = FileCache::new(project_dirs().cache_dir().into())?;

        // Request connection to remote host
        let mut conn = TcpStream::connect(addr)?;
        conn.set_nonblocking(true)?;
        let manifest = plugin_cache.manifest().keys().copied().collect();
        let req = ConnectionRequest::new(username, manifest);
        let req = serialize(&req).unwrap();
        conn.write_all(&req)?;

        // Receive response from server
        let mut recv_buf = AsyncBufferedReceiver::new();
        let response: ConnectionResponse;
        loop {
            match recv_buf.read(&mut conn)? {
                ReadState::Complete(data) => {
                    response = deserialize(std::io::Cursor::new(data))?;
                    break;
                }
                ReadState::Incomplete => {
                    // Don't busy the CPU too much while waiting for a response
                    std::thread::yield_now();
                }
                ReadState::Disconnected => bail!("Remote host hung up"),
                ReadState::Invalid => bail!("Invalid message from remote"),
            }
        }

        // Load needed plugins into memory
        let mut plugins = vec![];
        for (name, plugin) in response.plugins {
            let bytecode;
            match plugin {
                PluginData::Cached(digest) => {
                    let path = plugin_cache
                        .manifest()
                        .get(&digest)
                        .expect("Server did not send all plugins it was supposed to");
                    bytecode = std::fs::read(path)?;
                }
                PluginData::Download(data) => {
                    log::info!("Downloaded {}, saving...", name);
                    plugin_cache.add_file(&name, &data)?;
                    bytecode = data;
                }
            }

            plugins.push((name, bytecode));
        }

        // Set up engine and initialize plugins
        let mut engine = Engine::new(&plugins, Config { is_server: false })?;

        // Set up rendering
        let render = RenderPlugin::new(gl, &mut engine).context("Setting up render engine")?;

        let ui = OverlayUi::new(&mut engine);

        let gamepad = GamepadPlugin::new()?;

        // Initialize plugins AFTER we set up our plugins
        engine.init_plugins()?;

        Ok(Self {
            recv_buf,
            gamepad,
            conn,
            ui,
            engine,
            render,
        })
    }

    pub fn set_resolution(&mut self, width: u32, height: u32) {
        self.render.set_screen_size(width, height)
    }

    /// Synchronize with remote and with plugin hotloading
    pub fn download(&mut self) -> Result<()> {
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

                    // Load hotloaded plugins
                    for (name, bytecode) in recv.hotload {
                        log::info!("Reloading {}", name);
                        self.engine.reload(name, &bytecode)?;
                    }

                    // Receive remote messages
                    for msg in recv.messages {
                        self.engine.broadcast_local(msg);
                    }

                    // Synchronize ECS state
                    self.engine.ecs().import(
                        &Query::new().intersect::<Synchronized>(Access::Write),
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

    pub fn render_frame(&mut self, vr_view: Mat4, view_idx: usize) -> Result<()> {
        self.render.frame(&mut self.engine, vr_view, view_idx)
    }

    pub fn upload(&mut self) -> Result<()> {
        // Send message to server
        let msg = ClientToServer {
            messages: self.engine.network_inbox(),
        };

        self.conn.set_nonblocking(false)?;
        length_delimit_message(&msg, &mut self.conn)?;
        self.conn.flush()?;
        self.conn.set_nonblocking(true)?;

        Ok(())
    }

    fn engine(&mut self) -> &mut Engine {
        &mut self.engine
    }
}

fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("com", "ChatImproVR", "ChatImproVR")
        .expect("Failed to determine project dirs")
}

fn random_number() -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    RandomState::new().build_hasher().finish()
}
