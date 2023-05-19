use crate::{
    component_id,
    pcg::Pcg,
    prelude::*,
    serial::{
        deserialize, serialize, serialize_into, serialized_size, EcsData, ReceiveBuf, SendBuf,
    },
};
pub use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Defines the given structure to represent the state of a plugin (on either the **Client** or the
/// **Server**). Essentially defines the entry point for the plugin.
/// TODO: Probably rename this as PluginEntry or something.
pub trait UserState: Sized {
    /// Constructor for this state; called once before the **Init** stage.
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self;
}

/// A dummy UserState that doesn't do anything.
///
/// Useful if your plugin does not have any server state or any client state, e.g.
/// ```rust
/// use cimvr_engine_interface::prelude::*;
/// make_app_state!(MyClientState, DummyUserState)
/// ```
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

/// Basically main() for plugins; allows a struct implementing the [UserState](crate::plugin::UserState) trait to be the state and entry
/// point for the plugin.
///
/// Order matters to define which is the client state and which is the server state. Whatever goes first is the client state, and whatever goes second is the server state.
///
/// The Syntax is `make_app_state(ClientState, ServerState)`, in that order.
///
/// For example:
/// ```rust
/// struct MyClientState;
///
/// impl UserState for MyClientState {
///     fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
///         println!("Hello, world!");
///     }
/// }
///
/// make_app_state!(MyClientState, DummyUserState);
/// ```
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
            fn _cimvr_reserve(bytes: u32) -> u32 {
                // TODO: What if we fail?
                CTX.lock().unwrap().reserve(bytes) as _
            }

            /// Run internal code, returning pointer to the output buffer
            #[no_mangle]
            fn _cimvr_dispatch() -> *mut u8 {
                CTX.lock().unwrap().dispatch()
            }
        }
    };
}

/// Contains commands to be sent to the engine and received messages.
/// TODO: Find a better name for this lmao
#[derive(Serialize, Deserialize)]
pub struct EngineIo {
    /// Random number generator
    #[serde(skip)]
    pub(crate) pcg: Pcg,
    /// Sent commands
    pub(crate) commands: Vec<EcsCommand>,
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
    pub fn add_system(&mut self, callback: Callback<U>) -> SystemBuilder<U> {
        SystemBuilder {
            sched: self,
            desc: SystemDescriptor::default(),
            callback,
        }
    }
}

#[must_use = "Entities must be built with .build()"]
pub struct EntityBuilder<'io> {
    io: &'io mut EngineIo,
    entity: EntityId,
}

impl EntityBuilder<'_> {
    /// Add a component to the entity
    pub fn add_component<C: Component>(self, data: C) -> Self {
        self.io.add_component(self.entity, data);
        self
    }

    /// Build this entity, returning its id
    pub fn build(self) -> EntityId {
        self.entity
    }
}

#[must_use = "Systems must be built with .build()"]
pub struct SystemBuilder<'sched, U> {
    sched: &'sched mut EngineSchedule<U>,
    desc: SystemDescriptor,
    callback: Callback<U>,
}

impl<'a, U> SystemBuilder<'a, U> {
    /// Run the system during the specified Stage
    pub fn stage(mut self, stage: Stage) -> Self {
        self.desc.stage = stage;
        self
    }

    /// Query the given component and provide an access level to it.
    pub fn query(mut self, name: &'static str, query: Query) -> Self {
        self.desc.queries.insert(name.to_string(), query);
        self
    }

    /// Subscribe to the given channel by telling it which message type you want.
    pub fn subscribe<M: Message>(mut self) -> Self {
        self.desc.subscriptions.push(M::CHANNEL.into());
        self
    }

    /// Builds the system
    pub fn build(self) {
        self.sched.systems.push(self.desc);
        self.sched.callbacks.push(self.callback);
    }
}

impl<U: UserState> PluginState<U> {
    fn dispatch(&mut self, io: &mut EngineIo, ecs: EcsData, system_idx: usize) {
        // Call system function with user data
        let system = self.sched.callbacks[system_idx];

        // Get query results
        let query = self.sched.systems[system_idx].queries.clone();
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
    /// Called from _cimvr_reserve() oddly enough, because this structure manages memory.
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
    pub fn create_entity(&mut self) -> EntityBuilder {
        let entity = self.create_entity_internal();
        EntityBuilder { io: self, entity }
    }

    fn create_entity_internal(&mut self) -> EntityId {
        let id = EntityId(self.pcg.gen_u128());
        self.commands.push(EcsCommand::Create(id));
        id
    }

    /// Add a component to an entity
    /// You may also use this to update existing component data, but it's better to write to the
    /// query for large batches instead
    #[track_caller]
    pub fn add_component<C: Component>(&mut self, entity: EntityId, data: C) {
        let data = serialize(&data).expect("Failed to serialize component data");

        self.commands
            .push(EcsCommand::AddComponent(entity, component_id::<C>(), data));
    }

    /// Delete an entity and all of it's components
    pub fn remove_entity(&mut self, id: EntityId) {
        self.commands.push(EcsCommand::Delete(id));
    }

    /// Generate a pseudorandom number
    pub fn random(&mut self) -> u128 {
        self.pcg.gen_u128()
    }

    /// Read inbox for this message type
    pub fn inbox<M: Message>(&self) -> impl Iterator<Item = M> + '_ {
        self.inbox
            .get(&M::CHANNEL.into())
            .map(|v| v.as_slice())
            .unwrap_or_default()
            .iter()
            .map(|m| {
                deserialize(std::io::Cursor::new(&m.data)).expect("Failed to deserialize message")
            })
    }

    /// Read inbox for this message type, along with client sender information
    pub fn inbox_clients<M: Message>(&mut self) -> impl Iterator<Item = (ClientId, M)> + '_ {
        assert_eq!(
            M::CHANNEL.locality,
            Locality::Remote,
            "It makes no sense to use this method for local messages!"
        );

        self.inbox
            .entry(M::CHANNEL.into())
            .or_default()
            .iter()
            .map(|m| {
                let data = deserialize(std::io::Cursor::new(&m.data))
                    .expect("Failed to deserialize message");
                (m.client.unwrap(), data)
            })
    }

    /// Send a message
    pub fn send<M: Message>(&mut self, data: &M) {
        self.outbox.push(MessageData {
            channel: M::CHANNEL.into(),
            data: serialize(data).expect("Failed to serialize message data"),
            client: None,
        });
    }

    /// Send a message to a specific client
    pub fn send_to_client<M: Message>(&mut self, data: &M, client: ClientId) {
        self.outbox.push(MessageData {
            channel: M::CHANNEL.into(),
            data: serialize(data).expect("Failed to serialize message data"),
            client: Some(client),
        });
    }

    /// Get the first message on this channel, or return None
    pub fn inbox_first<M: Message>(&mut self) -> Option<M> {
        self.inbox
            .entry(M::CHANNEL.into())
            .or_default()
            .first()
            .map(|m| {
                deserialize(std::io::Cursor::new(&m.data)).expect("Failed to deserialize message")
            })
    }
}

impl UserState for DummyUserState {
    fn new(_: &mut EngineIo, _: &mut EngineSchedule<Self>) -> Self {
        Self
    }
}
