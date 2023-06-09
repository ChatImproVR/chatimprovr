use anyhow::Result;

use cimvr_engine::hotload::Hotloader;
use cimvr_engine::interface::prelude::{
    Access, ClientId, ConnectionRequest, ConnectionResponse, Connections, Digest, PluginData,
    Query, Synchronized,
};
use cimvr_engine::interface::serial::{deserialize, serialize, serialize_into};
use cimvr_engine::{calculate_digest, Config};
use cimvr_engine::{interface::system::Stage, network::*, Engine};

use std::time::Instant;
use std::{
    io::Write,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "ChatImproVR Server",
    about = "Headless server application for hosting the ChatImproVR metaverse"
)]
struct Opt {
    /// Bind address
    #[structopt(short, long, default_value = "0.0.0.0:5031")]
    bind: SocketAddr,

    /// Plugins
    plugins: Vec<PathBuf>,
}

fn main() -> Result<()> {
    // Parse args
    let args = Opt::from_args();
    let bind_addr = args.bind.clone();
    println!("Binding to {}", bind_addr);

    // Set up logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Set up engine and initialize plugins
    let hotload = Hotloader::new(&args.plugins)?;

    let plugins: Vec<(String, Vec<u8>)> = args
        .plugins
        .iter()
        .map(|path| {
            let name = path_to_plugin_name(&path);
            let bytecode = std::fs::read(path)?;
            Ok((name, bytecode))
        })
        .collect::<Result<_>>()?;

    let mut engine = Engine::new(&plugins, Config { is_server: true })?;
    engine.init_plugins()?;

    // Create a new thread for the connection listener
    let (conn_tx, conn_rx) = mpsc::channel();
    std::thread::spawn(move || connection_listener(bind_addr, conn_tx));

    let mut server = Server::new(conn_rx, engine, hotload, plugins);
    let target = Duration::from_millis(15);

    loop {
        let start = Instant::now();
        server.update()?;
        let elap = start.elapsed();

        if let Some(wait_time) = target.checked_sub(elap) {
            std::thread::sleep(wait_time);
        }
    }
}

/// Thread which listens for new connections and sends them to the given MPSC channel
/// Technically we could use a non-blocking connection accepter, but it was easier not to for now
fn connection_listener(
    addr: SocketAddr,
    conn_tx: Sender<(TcpStream, ConnectionRequest)>,
) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    loop {
        let (mut stream, addr) = listener.accept()?;
        let Ok(req) = deserialize::<_, ConnectionRequest>(&mut stream) else {
            log::warn!("Failed connection from {}; bad request", addr);
            continue;
        };

        if req.validate() {
            conn_tx.send((stream, req)).unwrap();
        }
    }
}

/// A single tracked connection
struct Connection {
    /// TCP stream
    stream: TcpStream,
    // /// Address
    // addr: SocketAddr,
    /// Message read buffer
    msg_buf: AsyncBufferedReceiver,
    /// Connection ID
    id: ClientId,
    /// Username
    username: String,
}

/// Server internals
struct Server {
    /// ChatImproVR engine
    engine: Engine,
    /// Incoming connections
    conn_rx: Receiver<(TcpStream, ConnectionRequest)>,
    /// Existing connections
    conns: Vec<Connection>,
    /// Code hotloading
    hotload: Hotloader,
    /// Client ID increment
    id_counter: u32,
    /// Currently loaded plugin bytecode. Can change during runtime,
    /// so we keep this in order to send it to new clients
    bytecode: Vec<(Digest, String, Vec<u8>)>,
}

impl Server {
    fn new(
        conn_rx: Receiver<(TcpStream, ConnectionRequest)>,
        engine: Engine,
        hotload: Hotloader,
        bytecode: Vec<(String, Vec<u8>)>,
    ) -> Self {
        let bytecode = bytecode
            .into_iter()
            .map(|(name, code)| (calculate_digest(&code), name, code))
            .collect();
        Self {
            bytecode,
            hotload,
            engine,
            conn_rx,
            conns: vec![],
            id_counter: 0,
        }
    }

    fn update(&mut self) -> Result<()> {
        // Check for hotloaded plugins
        let mut hotloaded = vec![];
        for path in self.hotload.hotload()? {
            log::info!("Reloading {}", path.display());
            let name = path_to_plugin_name(&path);
            let bytecode = std::fs::read(path)?;
            self.engine.reload(name.clone(), &bytecode)?;

            // Update bytecode on our side so that newly connected clients will have the current code
            let (entry_digest, entry_bytecode) = self
                .bytecode
                .iter_mut()
                .find_map(|(digest, plugin_name, code)| {
                    (plugin_name == &name).then(|| (digest, code))
                })
                .unwrap();
            *entry_digest = calculate_digest(&bytecode);
            *entry_bytecode = bytecode.clone();

            // Remember which plugins were hotloaded, so that we can send code to
            // the clients!
            hotloaded.push((name, bytecode));
        }

        let mut conns_tmp = vec![];

        // Check for new connections
        for (mut stream, req) in self.conn_rx.try_iter() {
            let addr = stream.peer_addr()?;

            // Create connection on our side
            log::info!("{} Connected from {}", req.username, addr);

            // Send plugins to client
            let mut response_plugins = vec![];
            for (digest, name, code) in &self.bytecode {
                // Only send plugin code that a given client does not already have!
                if req.plugin_manifest.contains(&digest) {
                    response_plugins.push((name.clone(), PluginData::Cached(*digest)));
                } else {
                    response_plugins.push((name.clone(), PluginData::Download(code.clone())));
                }
            }

            let resp = ConnectionResponse {
                plugins: response_plugins,
            };

            // Write response
            // TODO: Make this async - blocks whole server just to upload plugin data!
            if let Err(e) = length_delimit_message(&resp, &mut stream) {
                log::error!("Client connection failed; {:#}", e);
            } else {
                stream.set_nonblocking(true)?;
                // Remember connection on our side
                self.conns.push(Connection {
                    msg_buf: AsyncBufferedReceiver::new(),
                    stream,
                    username: req.username,
                    id: ClientId(self.id_counter),
                });
                self.id_counter += 1;
            }
        }

        // Read client messages
        for mut conn in self.conns.drain(..) {
            let keep_alive = loop {
                match conn.msg_buf.read(&mut conn.stream)? {
                    ReadState::Disconnected => {
                        log::info!("{} Disconnected", conn.username);
                        break false;
                    }
                    ReadState::Complete(buf) => {
                        let msgs: ClientToServer =
                            deserialize(std::io::Cursor::new(buf)).expect("Malformed message");
                        // Broadcast from client to server modules
                        for mut msg in msgs.messages {
                            // Set the client ID for each message(!)
                            msg.client = Some(conn.id);
                            self.engine.broadcast_local(msg);
                        }
                        continue;
                    }
                    ReadState::Invalid => {
                        log::error!("Invalid data on connection");
                        break true;
                    }
                    ReadState::Incomplete => {
                        break true;
                    }
                };
            };

            if keep_alive {
                conns_tmp.push(conn);
            }
        }

        // Send connection list
        self.engine.send(Connections {
            clients: conns_tmp
                .iter()
                .map(|c| cimvr_engine::interface::prelude::Connection {
                    id: c.id,
                    username: c.username.clone(),
                })
                .collect(),
        });

        // Execute update steps
        self.engine.dispatch(Stage::PreUpdate)?;
        self.engine.dispatch(Stage::Update)?;
        self.engine.dispatch(Stage::PostUpdate)?;

        // Gather current synchronized state
        let state = ServerToClient {
            ecs: self
                .engine
                .ecs()
                .export(&Query::new().intersect::<Synchronized>(Access::Read)),
            messages: self.engine.network_inbox(),
            hotload: hotloaded,
        };

        // Write header and serialize message

        // Broadcast to clients
        for mut conn in conns_tmp.drain(..) {
            // TODO: This is a dumb, slow way to do this lol
            // Only send message to the clients which they are destined for
            let mut state = state.clone();
            state.messages.retain(|m| match m.client {
                None => true,
                Some(outgoing) => outgoing == conn.id,
            });

            // Serialize message
            conn.stream.set_nonblocking(false)?;
            if let Err(e) = length_delimit_message(&state, &mut conn.stream) {
                log::error!("Error writing to stream; {:?}", e);
            } else {
                conn.stream.flush()?;
                conn.stream.set_nonblocking(true)?;
                self.conns.push(conn);
            }
        }

        Ok(())
    }
}

fn path_to_plugin_name(path: &Path) -> String {
    path.file_name().unwrap().to_str().unwrap().to_string()
}
