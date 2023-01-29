use cimvr_engine_interface::{dbg, make_app_state, prelude::*};
use serde::{Deserialize, Serialize};

struct ClientState {
    increment: i32,
}

struct ServerState;

make_app_state!(ClientState, ServerState);

// Message datatype
#[derive(Serialize, Deserialize, Debug)]
struct MyMessage {
    a: i32,
    b: f32,
}

impl Message for MyMessage {
    const CHANNEL: ChannelId = ChannelId {
        // Arbitrary! But must be random
        id: 232171203811109360315433038366538761455,
        // Sent to server
        locality: Locality::Remote,
    };
}

// Client code
impl UserState for ClientState {
    fn new(_io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        sched.add_system(Self::update, SystemDescriptor::new(Stage::Update));

        Self { increment: 0 }
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
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
        sched.add_system(
            Self::update,
            SystemDescriptor::new(Stage::Update).subscribe::<MyMessage>(),
        );

        Self
    }
}

impl ServerState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        for msg in io.inbox::<MyMessage>() {
            dbg!(msg);
        }
    }
}
