use std::sync::{Mutex, MutexGuard};

pub use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
    ecs::{Component, EntityId},
    serial::SystemDescriptor,
};

/// Full plugin context, contains user state and engine IO buffers
pub struct Context<U> {
    user: U,
    io: EngineIo,
    sched: EngineSchedule<U>,
}

/// System callable by the engine  
pub type Callback<UserState> = fn(&mut UserState, &mut EngineIo);

/// Basically main() for plugins; allows a struct implementing AppState to be the state and entry
/// point for the plugin
#[macro_export]
macro_rules! make_app_state {
    ($AppState:ident) => {
        static CTX: Lazy<Mutex<Context<$AppState>>> = Lazy::new(|| Mutex::new(Context::new()));

        /// Reserve internal memory for external writes
        #[no_mangle]
        fn reserve(bytes: u32) -> *mut u8 {
            // TODO: What if we fail?
            CTX.lock().unwrap().reserve(bytes)
        }

        /// Run internal code, returning pointer to the output buffer
        #[no_mangle]
        fn dispatch() -> *mut u8 {
            CTX.lock().unwrap().dispatch()
        }

        /// Externally inspectable engine version
        #[no_mangle]
        fn engine_version() -> u32 {
            0
        }
    };
}

/// Application state, defines a constructor with common engine interface in it
pub trait AppState: Sized {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self;
}

/// Contains the query result, and any received messages.
/// Also contains the commands to be sent to the engine, and lists the modified entities and
/// components therein
pub struct EngineIo {
    buf: Vec<u8>,
}

/// Scheduling of systems
/// Not a part of EngineIo, in order to prevent developers from attempting to add systems from
/// other systems (!)
pub struct EngineSchedule<U> {
    systems: Vec<SystemDescriptor>,
    callbacks: Vec<Callback<U>>,
}

impl<U> EngineSchedule<U> {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            callbacks: Vec::new(),
        }
    }

    pub fn add_system(&mut self, desc: SystemDescriptor, cb: Callback<U>) {
        self.systems.push(desc);
        self.callbacks.push(cb);
    }
}

impl<U: AppState> Context<U> {
    pub fn new() -> Self {
        let mut io = EngineIo::new();
        let mut sched = EngineSchedule::new();
        Self {
            user: U::new(&mut io, &mut sched),
            sched,
            io,
        }
    }

    pub fn reserve(&mut self, bytes: u32) -> *mut u8 {
        self.io.reserve(bytes);
        self.io.buf_ptr()
    }

    pub fn dispatch(&mut self) -> *mut u8 {
        // TODO: Read from own io buf, Dispatch

        self.io.buf_ptr()
    }
}

impl EngineIo {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn add_component<C: Component>(&self, entity: EntityId, data: C) {
        todo!()
    }

    pub fn create_entity(&self) -> EntityId {
        todo!()
    }

    pub fn remove_entity(&self, entity: EntityId) {
        todo!()
    }

    /// Reserves the given number of bytes for overwriting by the server
    pub fn reserve(&mut self, bytes: u32) {
        self.buf.clear();
        self.buf.resize(bytes as usize, 0);
    }

    pub fn buf_ptr(&mut self) -> *mut u8 {
        self.buf.as_mut_ptr()
    }
    //fn add_system(&mut self, system, callback: fn());
}
