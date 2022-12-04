use std::sync::{Mutex, MutexGuard};

pub use once_cell::sync::Lazy;

use crate::ecs::{Component, EntityId};

pub type Callback<UserState> = fn(&mut UserState, &mut EngineIo, &mut QueryResult);

pub struct EngineIo {
    buf: Vec<u8>,
    //commands: Vec<Command>,
}

pub struct QueryResult<'a> {
    io: &'a EngineIo,
    // query indexing stuff idk
    // impl iterator yea
}

pub struct Context<U> {
    user: U,
    callbacks: Vec<Callback<U>>,
    io: EngineIo,
    //_phantom: PhantomData<UserState>,
}

impl EngineIo {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }
}

impl<U> Context<U> {
    pub fn new(ctor: fn(&mut EngineIo) -> U) -> Self {
        let mut io = EngineIo::new();
        Self {
            user: ctor(&mut io),
            callbacks: vec![],
            io,
        }
    }

    pub fn reserve(&self, bytes: u32) -> *mut u8 {
        self.io.reserve(bytes)
    }

    pub fn dispatch(&self) -> *mut u8 {
        todo!()
    }
}

impl EngineIo {
    pub fn add_component<C: Component>(&self, entity: EntityId, data: C) {
        todo!()
    }

    pub fn create_entity(&self) -> EntityId {
        todo!()
    }

    pub fn remove_entity(&self, entity: EntityId) {
        todo!()
    }

    pub fn reserve(&self, bytes: u32) -> *mut u8 {
        todo!()
    }
    //fn add_system(&mut self, system, callback: fn());
}
