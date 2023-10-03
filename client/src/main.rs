extern crate glow as gl;

#[cfg(feature = "vr")]
extern crate openxr as xr;

use anyhow::{bail, format_err, Context, Result};
use cimvr_common::glam::Mat4;
use cimvr_common::InterdimensionalTravelRequest;
use cimvr_engine::hotload::Hotloader;
use cimvr_engine::interface::prelude::{
    Access, ClientId, ConnectionRequest, ConnectionResponse, PluginData, Query, Synchronized,
};
use cimvr_engine::interface::serial::{deserialize, serialize};
use cimvr_engine::network::{
    length_delimit_message, AsyncBufferedReceiver, ClientToServer, ReadState, ServerToClient,
};
use cimvr_engine::Engine;
use cimvr_engine::{calculate_digest, Config};
use directories::ProjectDirs;
use eframe::egui;
use gamepad::GamepadPlugin;
use plugin_cache::FileCache;
use render::RenderPlugin;
use std::collections::HashSet;
use std::io::Write;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "vr")]
mod vr;

mod desktop;
mod desktop_input;
mod gamepad;
mod plugin_cache;
mod render;

use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
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
    conn: ServerOrTcp,
    gamepad: GamepadPlugin,
}

enum ServerOrTcp {
    BuiltinServer {
        engine: Engine,
        plugins: Vec<(String, Vec<u8>)>,
    },
    Tcp {
        conn: TcpStream,
        recv_buf: AsyncBufferedReceiver,
    },
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
    pub fn new(gl: Arc<gl::Context>, login: LoginInfo) -> Result<Self> {
        // Set up plugin cache
        let mut plugin_cache = FileCache::new(project_dirs().cache_dir().into())?;

        // Request connection to remote host, uploading manifest of plugins
        // TODO: Replace the manifest with a plain ol HTTP cache
        #[cfg(not(feature = "embed-server"))]
        let mut conn = ServerOrTcp::connect_tcp(login.addr_with_port())?;

        #[cfg(feature = "embed-server")]
        let mut conn = ServerOrTcp::builtin_server(vec![
            (
                "camera".into(),
                include_bytes!("../../target/wasm32-unknown-unknown/release/camera.wasm").to_vec(),
            ),
            (
                "cube".into(),
                include_bytes!("../../target/wasm32-unknown-unknown/release/cube.wasm").to_vec(),
            ),
            /*
            (
                "ui_example".into(),
                include_bytes!("../../target/wasm32-unknown-unknown/release/ui_example.wasm").to_vec(),
            )*/
        ])?;

        let manifest = plugin_cache.manifest().keys().copied().collect();
        let conn_reqeust = ConnectionRequest::new(login.username, manifest);
        let response = conn.login(conn_reqeust)?;

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

        let gamepad = GamepadPlugin::new()?;

        // Set up interdimensional travel
        engine.subscribe::<InterdimensionalTravelRequest>();

        // Initialize plugins AFTER we set up our plugins
        engine.init_plugins()?;

        Ok(Self {
            gamepad,
            conn,
            engine,
            render,
        })
    }

    /// Synchronize with remote and with plugin hotloading
    pub fn download(&mut self) -> Result<()> {
        for recv in self.conn.download()? {
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

        Ok(())
    }

    pub fn prep_render(&mut self) -> Result<()> {
        self.render.handle_messages(&mut self.engine)
    }

    pub fn render_frame(&mut self, vr_view: Mat4, view_idx: usize) -> Result<()> {
        self.render.frame(&mut self.engine, vr_view, view_idx)
    }

    pub fn upload(&mut self) -> Result<()> {
        // Send message to server
        let msg = ClientToServer {
            messages: self.engine.network_inbox(),
        };

        self.conn.upload(msg)?;

        Ok(())
    }

    fn engine(&mut self) -> &mut Engine {
        &mut self.engine
    }

    fn travel_request(&mut self) -> Option<InterdimensionalTravelRequest> {
        self.engine().inbox().next()
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LoginInfo {
    pub address: String,
    pub username: String,
}

impl LoginInfo {
    /// Returns the address assigned, with the default port appended if not present
    pub fn addr_with_port(&self) -> String {
        let addr = self.address.clone();
        if addr.contains(':') {
            addr
        } else {
            addr + ":5031"
        }
    }
}

pub struct LoginFile {
    pub username: String,
    pub last_login_address: String,
    pub addresses: Vec<String>,
}

impl LoginFile {
    fn config_path() -> PathBuf {
        let proj = project_dirs();
        if !proj.config_dir().is_dir() {
            std::fs::create_dir_all(proj.config_dir()).unwrap();
        }
        proj.config_dir().join("login.conf")
    }

    pub fn save(&mut self) -> Result<()> {
        use std::fmt::Write;
        let mut s = String::new();
        writeln!(s, "{}", self.username)?;
        writeln!(s, "{}", self.last_login_address)?;

        for addr in &self.addresses {
            writeln!(s, "{}", addr)?;
        }

        std::fs::write(Self::config_path(), s)?;
        Ok(())
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        let mut inst = Self::default();

        let text: String;
        if config_path.is_file() {
            text = std::fs::read_to_string(config_path)?;
        } else {
            text = "".into();
        }

        let mut lines = text.lines().map(ToOwned::to_owned);

        if let Some(username) = lines.next() {
            inst.username = username;
        }

        if let Some(last_login_addr) = lines.next() {
            inst.last_login_address = last_login_addr;
        }

        for line in lines {
            inst.addresses.push(line);
        }

        Ok(inst)
    }
}

impl Default for LoginFile {
    fn default() -> Self {
        Self {
            username: LoginInfo::default().username,
            last_login_address: LoginInfo::default().address,
            addresses: Default::default(),
        }
    }
}

impl Default for LoginInfo {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            username: "Anon".to_string(),
        }
    }
}

impl Opt {
    fn login_info(&self) -> Result<LoginInfo> {
        let mut login_file = LoginFile::load()?;

        Ok(LoginInfo {
            username: self.username.clone().unwrap_or(login_file.username),
            address: self
                .connect
                .clone()
                .unwrap_or(login_file.last_login_address),
        })
    }
}

impl ServerOrTcp {
    const CLIENT_ID: ClientId = ClientId(0);

    pub fn connect_tcp<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        // Receive response from server
        let recv_buf = AsyncBufferedReceiver::new();
        let conn = TcpStream::connect(addr)?;
        Ok(Self::Tcp { conn, recv_buf })
    }

    pub fn builtin_server(plugins: Vec<(String, Vec<u8>)>) -> Result<Self> {
        let mut engine = Engine::new(&plugins, Config { is_server: true })?;
        engine.init_plugins()?;
        Ok(Self::BuiltinServer { engine, plugins })
    }

    pub fn login(&mut self, request: ConnectionRequest) -> Result<ConnectionResponse> {
        match self {
            Self::Tcp { conn, recv_buf } => {
                conn.set_nonblocking(true)?;
                let req = serialize(&request).unwrap();
                conn.write_all(&req)?;

                let response: ConnectionResponse;
                loop {
                    match recv_buf.read(&mut *conn)? {
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

                Ok(response)
            }
            Self::BuiltinServer { engine, plugins } => Ok(ConnectionResponse {
                plugins: plugins
                    .iter()
                    .map(|(name, data)| (name.clone(), PluginData::Download(data.clone())))
                    .collect(),
            }),
        }
    }

    pub fn download(&mut self) -> Result<Vec<ServerToClient>> {
        match self {
            Self::Tcp { conn, recv_buf } => {
                let mut msgs = vec![];
                const MAX_MESSAGES_PER_FRAME: usize = 1000;
                for _ in 0..MAX_MESSAGES_PER_FRAME {
                    match recv_buf.read(&mut *conn)? {
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
                            msgs.push(recv);
                        }
                    }
                }

                Ok(msgs)
            }
            Self::BuiltinServer { engine, plugins } => Ok(vec![ServerToClient {
                ecs: engine
                    .ecs()
                    .export(&Query::new().intersect::<Synchronized>(Access::Read)),
                messages: engine.network_inbox(),
                hotload: vec![],
            }]),
        }
    }

    pub fn upload(&mut self, msg: ClientToServer) -> Result<()> {
        match self {
            Self::Tcp { conn, .. } => {
                conn.set_nonblocking(false)?;
                length_delimit_message(&msg, &mut *conn)?;
                conn.flush()?;
                conn.set_nonblocking(true)?;
                Ok(())
            }
            ServerOrTcp::BuiltinServer { engine, .. } => {
                for mut msg in msg.messages {
                    msg.client = Some(Self::CLIENT_ID);
                    engine.broadcast_local(msg);
                }
                Ok(())
            }
        }
    }
}
