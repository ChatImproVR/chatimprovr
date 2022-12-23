use anyhow::Result;
use cimvr_engine::ecs::{query_ecs_data, Ecs};
use cimvr_engine::interface::prelude::{query, Access, EntityId, Synchronized};
use cimvr_engine::interface::serial::{
    deserialize, serialize, serialize_into, serialized_size, EcsData,
};
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
    let mut engine = Engine::new(&args.plugins, true)?;
    engine.init_plugins()?;

    // Create a new thread for the connection listener
    let (conn_tx, conn_rx) = mpsc::channel();
    std::thread::spawn(move || connection_listener(bind_addr, conn_tx));

    let mut server = Server::new(conn_rx, engine);
    let target = Duration::from_millis(50);

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
}

struct Server {
    engine: Engine,
    conn_rx: Receiver<(TcpStream, SocketAddr)>,
    conns: Vec<Connection>,
}

impl Server {
    fn new(conn_rx: Receiver<(TcpStream, SocketAddr)>, engine: Engine) -> Self {
        Self {
            engine,
            conn_rx,
            conns: vec![],
        }
    }

    fn update(&mut self) -> Result<()> {
        let mut conns_tmp = vec![];

        // Check for new connections
        for (stream, addr) in self.conn_rx.try_iter() {
            stream.set_nonblocking(true)?;
            log::info!("{} Connected", addr);
            self.conns.push(Connection {
                msg_buf: AsyncBufferedReceiver::new(),
                stream,
                addr,
            });
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
                    for msg in msgs.messages {
                        self.engine.broadcast(msg);
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

        // Execute update steps
        self.engine.dispatch(Stage::PreUpdate)?;
        self.engine.dispatch(Stage::Update)?;
        self.engine.dispatch(Stage::PostUpdate)?;

        // Gather current synchronized state
        let state = ServerToClient {
            ecs: self
                .engine
                .ecs()
                .export(&[query::<Synchronized>(Access::Read)]),
            messages: self.engine.network_inbox(),
        };

        // Write header and serialize message
        let mut msg = vec![];
        length_delmit_message(&state, std::io::Cursor::new(&mut msg))?;

        // Broadcast to clients
        for mut conn in conns_tmp.drain(..) {
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

        Ok(())
    }
}
