use std::collections::HashMap;

pub use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
    dbg,
    pcg::Pcg,
    prelude::*,
    serial::{deserialize, serialize, serialize_into, serialized_size, ReceiveBuf, SendBuf},
};

/// Full plugin context, contains user state and engine IO buffers
pub struct Context<U> {
    /// User-defined state
    user: Option<U>,
    /// Buffer for communication with host
    buf: Vec<u8>, // TODO: SAFETY: Make this buffer volatile?! Host writes to it externally...
    /// Callbacks for systems and their associated subscription parameters
    sched: EngineSchedule<U>,
}

/// System callable by the engine  
pub type Callback<UserState> = fn(&mut UserState, &mut NonQueryIo, &mut QueryResult);

/// Basically main() for plugins; allows a struct implementing AppState to be the state and entry
/// point for the plugin
#[macro_export]
macro_rules! make_app_state {
    ($AppState:ident) => {
        mod _ctx {
            // TODO: This is a stupid hack
            use super::$AppState;
            use cimvr_engine_interface::plugin::{Context, Lazy};
            use std::sync::Mutex;
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
        }
    };
}

/// Application state, defines a constructor with common engine interface in it
pub trait AppState: Sized {
    fn new(io: &mut NonQueryIo, sched: &mut EngineSchedule<Self>) -> Self;
}

/// Contains the query result, and any received messages.
/// Also contains the commands to be sent to the engine, and lists the modified entities and
/// components therein
/// TODO: Find a better name for this lmao
#[derive(Serialize, Deserialize)]
pub struct NonQueryIo {
    /// Random number generator
    #[serde(skip)]
    pub(crate) pcg: Pcg,
    /// Sent commands
    pub(crate) commands: Vec<EngineCommand>,
    /// Sent messages
    pub(crate) outbox: Vec<Message>,
    /// Inbox
    pub(crate) inbox: Inbox,
    /*
    /// Received messages, one array for each subscribed channel (in the same order)
    pub(crate) message_rx: Vec<Vec<Message>>,
    /// Sent messages
    pub(crate) message_tx: Vec<Message>,
    /// Subscriptions, for referencing to get recieved messages on a channel
    subscriptions: Vec<ChannelId>,
    */
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

    // TODO: Decide whether ECS data is flushed to the engine in between!
    /// Contract: Systems within the same stage are executed in the order in which they are added
    /// by this function.
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
            sched: EngineSchedule::new(),
            buf: vec![],
        }
    }

    /// Entry point for user code
    pub fn dispatch(&mut self) -> *mut u8 {
        // Deserialize state from server
        let recv: ReceiveBuf =
            deserialize(std::io::Cursor::new(&self.buf)).expect("Failed to decode host message");

        let mut io = NonQueryIo::new(recv.inbox);

        if let Some(system_idx) = recv.system {
            // Call system function with user data
            let user = self
                .user
                .as_mut()
                .expect("Attempted to call system before initialization");
            let system = self.sched.callbacks[system_idx];

            // Get query results
            let query = self.sched.systems[system_idx].query.clone();
            let mut query_result = QueryResult::new(recv.ecs, query);

            // Run the user's system
            system(user, &mut io, &mut query_result);

            io.commands.extend(query_result.commands);
        } else {
            // Initialize plugin internals
            self.user = Some(U::new(&mut io, &mut self.sched));
        }

        // Write return state
        let send = SendBuf {
            commands: std::mem::take(&mut io.commands),
            outbox: std::mem::take(&mut io.outbox),
            systems: self.sched.systems.clone(),
        };
        let len: u32 = serialized_size(&send).expect("Failed to get size of host message") as u32;

        // Write header
        self.buf.clear();
        self.buf.extend(len.to_le_bytes());

        // Write data
        serialize_into(&mut self.buf, &send).expect("Failed to encode host message");

        // Return buffer pointer
        self.buf.as_mut_ptr()
    }

    /// Reserves the given number of bytes for overwriting by the server
    pub fn reserve(&mut self, bytes: u32) -> *mut u8 {
        self.buf.clear();
        self.buf.resize(bytes as usize, 0);
        self.buf.as_mut_ptr()
    }
}

impl NonQueryIo {
    pub fn new(inbox: Inbox) -> Self {
        Self {
            commands: vec![],
            pcg: Pcg::new(),
            outbox: vec![],
            inbox,
        }
    }

    pub fn add_component<C: Component>(&mut self, entity: EntityId, data: &C) {
        let data = serialize(data).expect("Failed to serialize component data");

        // Sanity check
        assert_eq!(
            data.len(),
            usize::from(C::ID.size),
            "Component size mismatch; ComponentId prescribes {} but serialize reports {}",
            C::ID.size,
            data.len(),
        );

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

    pub fn inbox(&self) -> &Inbox {
        &self.inbox
    }

    pub fn send(&mut self, channel: ChannelId, data: Vec<u8>) {
        self.outbox.push(Message { channel, data });
    }
}
