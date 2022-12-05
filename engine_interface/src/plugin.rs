use std::sync::{Mutex, MutexGuard};

pub use once_cell::sync::Lazy;

use crate::ecs::{Component, EntityId, Query, QueryResult};

pub type Callback<UserState> = fn(&mut UserState, &mut EngineIo);

/// Contains the query result, and any received messages. 
/// Also contains the commands to be sent to the engine, and lists the modified entities and
/// components therein
pub struct EngineIo {
    buf: Vec<u8>,
    commands: Vec<EngineCommand>,
}

enum EngineCommand {
    Delete(EntityId),
}

pub struct EngineSchedule<U> {
    systems: Vec<(Query, Callback<U>)>,
}

pub struct Context<U> {
    user: U,
    io: EngineIo,
    sched: EngineSchedule<U>,
}

impl<U> EngineSchedule<U> {
    pub fn new() -> Self {
        Self { 
            systems: Vec::new(),
        }
    }

    pub fn add_system(&mut self, query: Query, cb: Callback<U>) {
        self.systems.push((query, cb));
    }
}


impl<U> Context<U> {
    pub fn new(ctor: fn(&mut EngineIo, &mut EngineSchedule<U>) -> U) -> Self {
        let mut io = EngineIo::new();
        let mut sched = EngineSchedule::new();
        Self {
            user: ctor(&mut io, &mut sched),
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
        Self { 
            buf: Vec::new(),
            commands: Vec::new(),
        }
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
