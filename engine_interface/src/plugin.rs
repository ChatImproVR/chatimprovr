use std::sync::{Mutex, MutexGuard};

pub use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
    ecs::{Component, EntityId},
    pcg::Pcg,
    prelude::{setup_panic, EngineCommand},
    serial::{serialize, EcsData, SystemDescriptor},
};

/// Full plugin context, contains user state and engine IO buffers
pub struct Context<U> {
    /// User-defined state
    user: Option<U>,
    /// Ecs state, commands
    io: EngineIo,
    /// Buffer for communication with host
    buf: Vec<u8>,
    /// Callbacks for systems and their associated subscription parameters
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
        fn _reserve(bytes: u32) -> *mut u8 {
            // TODO: What if we fail?
            CTX.lock().unwrap().reserve(bytes)
        }

        /// Run internal code, returning pointer to the output buffer
        #[no_mangle]
        fn _dispatch() -> *mut u8 {
            CTX.lock().unwrap().dispatch()
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
    pub(crate) pcg: Pcg,
    pub(crate) ecs: EcsData,
    pub(crate) commands: Vec<EngineCommand>,
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
    /// Creates context, but don't set up usercode yet since we're not in _dispatch(),
    /// and that means that the engine would never see our output.
    /// Called from _reserve() oddly enough, because this structure manages memory.
    pub fn new() -> Self {
        setup_panic();

        Self {
            user: None,
            io: EngineIo::new(),
            sched: EngineSchedule::new(),
            buf: vec![],
        }
    }

    /// Entry point for user code
    pub fn dispatch(&mut self) -> *mut u8 {
        // Initialize user code if not already present
        // We do this BEFORE EngineIo is actually filled, so that it doesn't have any query data
        let user = self
            .user
            .get_or_insert_with(|| U::new(&mut self.io, &mut self.sched));

        self.buf.as_mut_ptr()
    }

    /// Reserves the given number of bytes for overwriting by the server
    pub fn reserve(&mut self, bytes: u32) -> *mut u8 {
        self.buf.clear();
        self.buf.resize(bytes as usize, 0);
        self.buf.as_mut_ptr()
    }
}

impl EngineIo {
    pub fn new() -> Self {
        Self {
            ecs: EcsData::default(),
            commands: vec![],
            pcg: Pcg::new(),
        }
    }

    pub fn add_component<C: Component>(&mut self, entity: EntityId, data: &C) {
        let data = serialize(data).expect("Failed to serialize component data");
        self.commands
            .push(EngineCommand::AddComponent(entity, C::ID, data));
    }

    pub fn create_entity(&mut self) -> EntityId {
        let id = EntityId(self.pcg.gen_u128());
        self.commands.push(EngineCommand::Create(id));
        id
    }

    pub fn remove_entity(&mut self, id: EntityId) {
        self.commands.push(EngineCommand::Delete(id));
    }

    pub fn random(&mut self) -> u32 {
        self.pcg.gen_u32()
    }
}
