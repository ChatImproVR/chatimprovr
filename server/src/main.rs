use anyhow::Result;

use cimvr_engine::hotload::Hotloader;
use cimvr_engine::interface::prelude::{
    Access, ClientId, ConnectionRequest, Connections, QueryComponent, Synchronized,
};
use cimvr_engine::interface::serial::deserialize;
use cimvr_engine::Config;
use cimvr_engine::{interface::system::Stage, network::*, Engine};

use std::time::Instant;
use std::{
    io::Write,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use std::path::PathBuf;
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
    let mut engine = Engine::new(&args.plugins, Config { is_server: true })?;
    engine.init_plugins()?;

    // Create a new thread for the connection listener
    let (conn_tx, conn_rx) = mpsc::channel();
    std::thread::spawn(move || connection_listener(bind_addr, conn_tx));

    let mut server = Server::new(conn_rx, engine, hotload);
    let target = Duration::from_millis(25);

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
fn connection_listener(addr: SocketAddr, conn_tx: Sender<(TcpStream, String)>) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    loop {
        let (mut stream, addr) = listener.accept()?;
        let Ok(req) = deserialize::<_, ConnectionRequest>(&mut stream) else {
            log::warn!("Failed connection from {}; bad request", addr);
            continue;
        };

        conn_tx.send((stream, req.username)).unwrap();
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
    /// Incoming connections (Socket, Username)
    conn_rx: Receiver<(TcpStream, String)>,
    /// Existing connections
    conns: Vec<Connection>,
    /// Code hotloading
    hotload: Hotloader,
    /// Client ID increment
    id_counter: u32,
}

impl Server {
    fn new(conn_rx: Receiver<(TcpStream, String)>, engine: Engine, hotload: Hotloader) -> Self {
        Self {
            hotload,
            engine,
            conn_rx,
            conns: vec![],
            id_counter: 0,
        }
    }

    fn update(&mut self) -> Result<()> {
        // Check for hotloaded plugins
        for path in self.hotload.hotload()? {
            log::info!("Reloading {}", path.display());
            self.engine.reload(path)?;
        }

        let mut conns_tmp = vec![];

        // Check for new connections
        for (stream, username) in self.conn_rx.try_iter() {
            stream.set_nonblocking(true)?;
            let addr = stream.peer_addr()?;
            log::info!("{} Connected from {}", username, addr);
            self.conns.push(Connection {
                msg_buf: AsyncBufferedReceiver::new(),
                stream,
                username,
                id: ClientId(self.id_counter),
            });
            self.id_counter += 1;
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
                .export(&[QueryComponent::new::<Synchronized>(Access::Read)]),
            messages: self.engine.network_inbox(),
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
            if let Err(e) = length_delmit_message(&state, &mut conn.stream) {
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
