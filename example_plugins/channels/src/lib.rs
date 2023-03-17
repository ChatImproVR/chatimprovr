use cimvr_engine_interface::{dbg, make_app_state, pkg_namespace, prelude::*};
use serde::{Deserialize, Serialize};

struct ClientState {
    increment: i32,
}

struct ServerState;

make_app_state!(ClientState, ServerState);

// It's important to make sure your package name is UNIQUE if you use this macro.
/// Message datatype
/// Implements Serialize and Deserialize, making it compatible with the Message trait. We
/// derive the Message trait, with locality "Remote" because we want this message sent
/// server-side.
#[derive(Message, Serialize, Deserialize, Debug)]
#[locality("Remote")]
struct MyMessage {
    a: i32,
    b: f32,
}

// Client code
impl UserState for ClientState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Schedule the update() system to run every Update
        sched.add_system(Self::update, SystemDescriptor::new(Stage::Update));

        Self { increment: 0 }
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        // Send a message to the server each frame
        io.send(&MyMessage {
            a: self.increment,
            b: self.increment as f32,
        });

        self.increment += 1;
    }
}

// Server code
impl UserState for ServerState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        // Schedule the update() system to run every Update,
        // and allow it to receive the MyMessage message
        sched.add_system(
            Self::update,
            SystemDescriptor::new(Stage::Update).subscribe::<MyMessage>(),
        );

        Self
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        // Dump both the message AND the client that sent the message to the console
        for (client, msg) in io.inbox_clients::<MyMessage>() {
            dbg!((client, msg));
        }
    }
}
