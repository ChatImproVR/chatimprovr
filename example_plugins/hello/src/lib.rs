use cimvr_engine_interface::{make_app_state, prelude::*, println};

// All state associated with client-side behaviour
struct ClientState;

impl UserState for ClientState {
    // Implement a constructor
    fn new(_io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        // NOTE: We are using the println defined by cimvr_engine_interface here, NOT the standard library!
        println!("Hello, client!");
        Self
    }
}

// All state associated with server-side behaviour
struct ServerState;

impl UserState for ServerState {
    // Implement a constructor
    fn new(_io: &mut EngineIo, _sched: &mut EngineSchedule<Self>) -> Self {
        println!("Hello, server!");
        Self
    }
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, ServerState);
