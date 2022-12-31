use anyhow::Result;
use cimvr_common::FrameTime;
use cimvr_engine::ecs::{query_ecs_data, Ecs};
use cimvr_engine::hotload::Hotloader;
use cimvr_engine::interface::prelude::{
    Access, ClientId, Connections, EntityId, QueryComponent, Synchronized,
};
use cimvr_engine::interface::serial::{
    deserialize, serialize, serialize_into, serialized_size, EcsData,
};
use cimvr_engine::Config;
use cimvr_engine::{interface::system::Stage, network::*, Engine};

use std::time::Instant;
use std::{
    io::{self, Write},
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
fn connection_listener(addr: SocketAddr, conn_tx: Sender<(TcpStream, SocketAddr)>) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    loop {
        conn_tx.send(listener.accept()?).unwrap();
    }
}

struct Connection {
    stream: TcpStream,
    addr: SocketAddr,
    msg_buf: AsyncBufferedReceiver,
    id: ClientId,
}

struct Server {
    engine: Engine,
    conn_rx: Receiver<(TcpStream, SocketAddr)>,
    conns: Vec<Connection>,
    hotload: Hotloader,
    start_time: Instant,
    last_frame: Instant,
    id_counter: u32,
}

impl Server {
    fn new(conn_rx: Receiver<(TcpStream, SocketAddr)>, engine: Engine, hotload: Hotloader) -> Self {
        Self {
            hotload,
            engine,
            last_frame: Instant::now(),
            start_time: Instant::now(),
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
        for (stream, addr) in self.conn_rx.try_iter() {
            stream.set_nonblocking(true)?;
            log::info!("{} Connected", addr);
            self.conns.push(Connection {
                msg_buf: AsyncBufferedReceiver::new(),
                stream,
                addr,
                id: ClientId(self.id_counter),
            });
            self.id_counter += 1;
        }

        // Read client messages
        for mut conn in self.conns.drain(..) {
            match conn.msg_buf.read(&mut conn.stream)? {
                ReadState::Disconnected => {
                    log::info!("{} Disconnected", conn.addr);
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

                    conns_tmp.push(conn);
                }
                ReadState::Invalid => {
                    log::error!("Invalid data on connection");
                    conns_tmp.push(conn);
                }
                ReadState::Incomplete => {
                    conns_tmp.push(conn);
                }
            };
        }

        // Send frame timing
        self.engine.send(FrameTime {
            time: self.start_time.elapsed().as_secs_f32(),
            delta: self.last_frame.elapsed().as_secs_f32(),
        });

        // Send connection list
        self.engine.send(Connections {
            clients: conns_tmp.iter().map(|c| c.id).collect(),
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
            let mut msg = vec![];
            length_delmit_message(&state, std::io::Cursor::new(&mut msg))?;

            match conn.stream.write_all(&msg) {
                Ok(_) => self.conns.push(conn),
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock => self.conns.push(conn),
                    io::ErrorKind::BrokenPipe
                    | io::ErrorKind::ConnectionReset
                    | io::ErrorKind::ConnectionAborted => {
                        log::info!("{} Disconnected; {:?}", conn.addr, e);
                    }
                    _ => return Err(e.into()),
                },
            }
        }

        self.last_frame = Instant::now();

        Ok(())
    }
}
