pub mod ecs;
pub mod network;
pub mod plugin;
use std::{collections::HashMap, path::PathBuf};

use anyhow::{Ok, Result};
pub use cimvr_engine_interface as interface;
use ecs::{apply_ecs_commands, query_ecs_data, Ecs};
use interface::{
    prelude::*,
    serial::{deserialize, serialize, EcsData, ReceiveBuf},
    system::Stage,
};
use plugin::Plugin;

// Keep the ECS in an Arc, so that it may be read simultaneously

/// Plugin state, plugin code, ECS state, messaging machinery, and more
pub struct Engine {
    /// WASM engine
    _wasm: wasmtime::Engine,
    /// Plugin states
    plugins: Vec<PluginState>,
    /// Entity and Component data
    ecs: Ecs,
    /// Message distribution indices, maps (channel id) -> (plugin index, system index)
    indices: HashMap<ChannelId, Vec<(usize, usize)>>,
    /// Host inboxes
    external_inbox: Inbox,
    /// Network inbox; messages to be sent from plugins to the remote(s)
    network_inbox: Vec<MessageData>,
    /// Am I a server?
    is_server: bool,
}

/// Plugin management structure
struct PluginState {
    /// Plugin code and interface
    code: Plugin,
    /// Systems on this plugin
    systems: Vec<SystemDescriptor>,
    /// Message inboxes, one for each system
    inbox: Vec<HashMap<ChannelId, Vec<MessageData>>>,
    // TODO: Make this Vec<Arc<Message>>? Faster! (No unnecessary copying)
    /// Message outbox
    outbox: Vec<MessageData>,
}

impl PluginState {
    pub fn new(code: Plugin) -> Self {
        PluginState {
            code,
            outbox: vec![],
            systems: vec![],
            inbox: Default::default(),
        }
    }
}

impl Engine {
    /// Load plugins at the given paths
    pub fn new(plugins: &[PathBuf], is_server: bool) -> Result<Self> {
        let wasm = wasmtime::Engine::new(&Default::default())?;
        let plugins: Vec<PluginState> = plugins
            .iter()
            .map(|p| Plugin::new(&wasm, p).map(PluginState::new))
            .collect::<Result<_>>()?;
        let ecs = Ecs::new();

        Ok(Self {
            _wasm: wasm,
            indices: HashMap::new(),
            plugins,
            ecs,
            external_inbox: HashMap::new(),
            network_inbox: vec![],
            is_server,
        })
    }

    /// Initialize plugin code. Must be called at least once!
    /// This is seperate from the constructor so that you may differentiate between loading errors
    /// and init errors, and also to allow you to decide when plugin code actually begins executing.
    pub fn init_plugins(&mut self) -> Result<()> {
        // Dispatch all plugins
        for (plugin_idx, plugin) in self.plugins.iter_mut().enumerate() {
            // Dispatch init signal
            let send = ReceiveBuf {
                system: None,
                inbox: Default::default(),
                ecs: EcsData::default(),
                is_server: self.is_server,
            };
            let recv = plugin.code.dispatch(&send)?;

            // Apply ECS commands
            apply_ecs_commands(&mut self.ecs, &recv.commands)?;

            // Setup message indices for each system
            for (sys_idx, sys) in recv.systems.iter().enumerate() {
                // Set up lookup table
                for &channel in &sys.subscriptions {
                    self.indices
                        .entry(channel)
                        .or_default()
                        .push((plugin_idx, sys_idx));
                }

                // Initialize system's inbox
                plugin.inbox.push(HashMap::new());
            }

            // Set up schedule, send first messages
            plugin.systems = recv.systems;
            plugin.outbox = recv.outbox;
        }

        // Distribute messages
        self.propagate();

        Ok(())
    }

    /// Dispatch plugin code on the given stage
    pub fn dispatch(&mut self, stage: Stage) -> Result<()> {
        // Run plugins
        for plugin in &mut self.plugins {
            for (system_idx, system) in plugin.systems.iter().enumerate() {
                // Filter to the requested stage
                if system.stage != stage {
                    continue;
                }

                // Query ECS
                let ecs_data = query_ecs_data(&mut self.ecs, &system.query)?;

                // Write input data
                let recv_buf = ReceiveBuf {
                    system: Some(system_idx),
                    inbox: std::mem::take(&mut plugin.inbox[system_idx]),
                    is_server: self.is_server,
                    ecs: ecs_data,
                };

                // Run plugin
                let ret = plugin.code.dispatch(&recv_buf)?;

                // Write back to ECS
                // TODO: Defer this? It's currently in Arbitrary order!
                apply_ecs_commands(&mut self.ecs, &ret.commands)?;

                // Receive outbox
                plugin.outbox = ret.outbox;
            }
        }

        // Distribute messages
        self.propagate();

        Ok(())
    }

    /// Propagate messages from plugin outboxes
    fn propagate(&mut self) {
        for i in 0..self.plugins.len() {
            for msg in std::mem::take(&mut self.plugins[i].outbox) {
                self.broadcast(msg);
            }
        }
    }

    /// Broadcast the message locally, without checkint to see if it's marked with local locality
    pub fn broadcast_local(&mut self, msg: MessageData) {
        if let Some(destinations) = self.indices.get(&msg.channel) {
            for (plugin_idx, system_idx) in destinations {
                self.plugins[*plugin_idx].inbox[*system_idx]
                    .entry(msg.channel)
                    .or_default()
                    .push(msg.clone());
            }
        } else {
            //log::warn!("Message on channel {:?} has no destination", msg.channel,);
        }

        if let Some(inbox) = self.external_inbox.get_mut(&msg.channel) {
            inbox.push(msg.clone());
        }
    }

    // TODO: Find a better name for this
    /// Broadcast message to relevant destinations
    pub fn broadcast(&mut self, msg: MessageData) {
        match msg.channel.locality {
            Locality::Local => self.broadcast_local(msg),
            Locality::Remote => {
                self.network_inbox.push(msg);
            }
        }
    }

    /// Access ECS data
    pub fn ecs(&mut self) -> &mut Ecs {
        &mut self.ecs
    }

    /// Subscribe to the given channel
    pub fn subscribe<M: Message>(&mut self) {
        self.external_inbox.entry(M::CHANNEL).or_default();
    }

    /// Drain messages from the given channel
    pub fn inbox<M: Message>(&mut self) -> impl Iterator<Item = M> + '_ {
        self.external_inbox
            .get_mut(&M::CHANNEL)
            .expect("Attempted to access a channel we haven't subscribed to")
            .drain(..)
            .map(|msg| {
                deserialize(std::io::Cursor::new(msg.data)).expect("Failed to decode message")
            })
    }

    /// Drain all network messages
    pub fn network_inbox(&mut self) -> Vec<MessageData> {
        std::mem::take(&mut self.network_inbox)
    }

    /// Broadcast a local message
    pub fn send<M: Message>(&mut self, data: M) {
        self.broadcast(MessageData {
            channel: M::CHANNEL,
            data: serialize(&data).expect("Failed to serialize message"),
            client: None,
        });
    }
}
