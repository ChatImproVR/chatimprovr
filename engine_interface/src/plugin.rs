use crate::{
    pcg::Pcg,
    prelude::*,
    serial::{
        deserialize, serialize, serialize_into, serialized_size, EcsData, ReceiveBuf, SendBuf,
    },
};
pub use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Application state, defines a constructor with common engine interface in it
pub trait UserState: Sized {
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self;
}

/// I'm dummy
/// Useful if you want client state but don't care about server state or vice versa
pub struct DummyUserState;

/// Full plugin context, contains user state and engine IO buffers
pub struct Context<C, S> {
    /// User-defined state
    user: Option<ClientOrServerState<C, S>>,
    /// Buffer for communication with host
    buf: Vec<u8>, // TODO: SAFETY: Make this buffer volatile?! Host writes to it externally...
}

/// Stores client or server specific state, callbacks
struct PluginState<U> {
    /// Callbacks for systems and their associated subscription parameters
    sched: EngineSchedule<U>,
    /// User state
    user: U,
}

/// System callable by the engine  
pub type Callback<U> = fn(&mut U, &mut EngineIo, &mut QueryResult);

/// User state tracking
enum ClientOrServerState<ClientState, ServerState> {
    Client(PluginState<ClientState>),
    Server(PluginState<ServerState>),
}

/// Basically main() for plugins; allows a struct implementing AppState to be the state and entry
/// point for the plugin
/// Syntax is `make_app_state(ClientState, ServerState)`
/// Use an empty struct or enum if you have no need for server state!
#[macro_export]
macro_rules! make_app_state {
    ($ClientState:ident, $ServerState:ident) => {
        mod _ctx {
            // TODO: This is a stupid hack
            use super::{$ClientState, $ServerState};
            use cimvr_engine_interface::plugin::{Context, Lazy};
            use std::sync::Mutex;
            static CTX: Lazy<Mutex<Context<$ClientState, $ServerState>>> =
                Lazy::new(|| Mutex::new(Context::new()));

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

/// Contains the query result, and any received messages.
/// Also contains the commands to be sent to the engine, and lists the modified entities and
/// components therein
/// TODO: Find a better name for this lmao
#[derive(Serialize, Deserialize)]
pub struct EngineIo {
    /// Random number generator
    #[serde(skip)]
    pub(crate) pcg: Pcg,
    /// Sent commands
    pub(crate) commands: Vec<EngineCommand>,
    /// Sent messages
    pub(crate) outbox: Vec<MessageData>,
    /// Inbox
    pub(crate) inbox: Inbox,
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

impl<U: UserState> PluginState<U> {
    fn dispatch(&mut self, io: &mut EngineIo, ecs: EcsData, system_idx: usize) {
        // Call system function with user data
        let system = self.sched.callbacks[system_idx];

        // Get query results
        let query = self.sched.systems[system_idx].query.clone();
        let mut query_result = QueryResult::new(ecs, query);

        // Run the user's system
        system(&mut self.user, io, &mut query_result);

        io.commands.extend(query_result.commands);
    }
}

impl<U: UserState> PluginState<U> {
    fn new(io: &mut EngineIo) -> Self {
        let mut sched = EngineSchedule::new();
        let user = U::new(io, &mut sched);
        Self { user, sched }
    }
}

impl<C: UserState, S: UserState> Context<C, S> {
    /// Creates context, but don't set up usercode yet since we're not in _dispatch(),
    /// and that means that the engine would never see our output.
    /// Called from _reserve() oddly enough, because this structure manages memory.
    pub fn new() -> Self {
        setup_panic();

        Self {
            user: None,
            buf: vec![],
        }
    }

    /// Entry point for user code
    pub fn dispatch(&mut self) -> *mut u8 {
        // Deserialize state from server
        let recv: ReceiveBuf =
            deserialize(std::io::Cursor::new(&self.buf)).expect("Failed to decode host message");

        let mut io = EngineIo::new(recv.inbox);

        if let (Some(sys_idx), Some(user)) = (recv.system, self.user.as_mut()) {
            // Dispatch plugin code
            match (recv.is_server, user) {
                (true, ClientOrServerState::Server(s)) => s.dispatch(&mut io, recv.ecs, sys_idx),
                (false, ClientOrServerState::Client(s)) => s.dispatch(&mut io, recv.ecs, sys_idx),
                _ => panic!("Are we a client or server plugin? Choose one!"),
            }
        } else {
            // Initialize plugin internals
            let user = match recv.is_server {
                true => ClientOrServerState::Server(PluginState::new(&mut io)),
                false => ClientOrServerState::Client(PluginState::new(&mut io)),
            };
            self.user = Some(user);
        }

        // Gather systems (should never change after init!)
        let systems = match self.user.as_ref().unwrap() {
            ClientOrServerState::Client(c) => c.sched.systems.clone(),
            ClientOrServerState::Server(s) => s.sched.systems.clone(),
        };

        // Write return state
        let send = SendBuf {
            commands: std::mem::take(&mut io.commands),
            outbox: std::mem::take(&mut io.outbox),
            systems,
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

impl EngineIo {
    pub(crate) fn new(inbox: Inbox) -> Self {
        Self {
            commands: vec![],
            pcg: Pcg::new(),
            outbox: vec![],
            inbox,
        }
    }

    /// Create an entity
    pub fn create_entity(&mut self) -> EntityId {
        let id = EntityId(self.pcg.gen_u128());
        self.commands.push(EngineCommand::Create(id));
        id
    }

    /// Add a component to an entity
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

    /// Delete an entity and all of it's components
    pub fn remove_entity(&mut self, id: EntityId) {
        self.commands.push(EngineCommand::Delete(id));
    }

    /// Generate a pseudorandom number
    pub fn random(&mut self) -> u32 {
        self.pcg.gen_u32()
    }

    /// Read inbox for this message type
    pub fn inbox<M: Message>(&mut self) -> impl Iterator<Item = M> + '_ {
        self.inbox.entry(M::CHANNEL).or_default().iter().map(|m| {
            deserialize(std::io::Cursor::new(&m.data)).expect("Failed to deserialize message")
        })
    }

    /// Send a message
    pub fn send<M: Message>(&mut self, data: &M) {
        self.outbox.push(MessageData {
            channel: M::CHANNEL,
            data: serialize(data).expect("Failed to serialize message data"),
            client: None,
        });
    }

    /// Get the first message on this channel, or return None
    pub fn inbox_first<M: Message>(&mut self) -> Option<M> {
        self.inbox.entry(M::CHANNEL).or_default().first().map(|m| {
            deserialize(std::io::Cursor::new(&m.data)).expect("Failed to deserialize message")
        })
    }
}

impl UserState for DummyUserState {
    fn new(_: &mut EngineIo, _: &mut EngineSchedule<Self>) -> Self {
        crate::println!("I'm dummy :3");
        Self
    }
}
