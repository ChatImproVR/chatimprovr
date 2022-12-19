pub mod ecs;
pub mod plugin;
use std::{collections::HashMap, path::PathBuf};

use anyhow::{Ok, Result};
pub use cimvr_engine_interface as interface;
use ecs::Ecs;
use interface::{
    prelude::*,
    serial::{deserialize, serialize, EcsData, ReceiveBuf, SendBuf},
    system::Stage,
};
use plugin::Plugin;

// Keep the ECS in an Arc, so that it may be read simultaneously

/// Plugin state, plugin code, ECS state, messaging machinery, and more
pub struct Engine {
    _wasm: wasmtime::Engine,
    plugins: Vec<PluginState>,
    ecs: Ecs,
    /// Message distribution indices, maps channel id -> plugin indices
    indices: HashMap<ChannelId, Vec<usize>>,
    /// User inboxes
    external_inbox: Inbox,
}

/// Plugin management structure
struct PluginState {
    /// Plugin code and interface
    code: Plugin,
    /// Systems on this plugin
    systems: Vec<SystemDescriptor>,
    /// Message inbox
    inbox: HashMap<ChannelId, Vec<MessageData>>,
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
            inbox: HashMap::default(),
        }
    }
}

impl Engine {
    /// Load plugins at the given paths
    pub fn new(plugins: &[PathBuf]) -> Result<Self> {
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
                inbox: std::mem::take(&mut plugin.inbox),
                ecs: EcsData::default(),
            };
            let recv = plugin.code.dispatch(&send)?;

            // Apply ECS commands
            apply_ecs_updates(&mut self.ecs, &recv)?;

            // Setup message indices
            for sys in &recv.systems {
                for &channel in &sys.subscriptions {
                    self.indices.entry(channel).or_default().push(plugin_idx);
                }
            }

            // Set up schedule, send first messages
            plugin.systems = recv.systems;
            plugin.outbox = recv.outbox;
        }

        // TODO: Panic if called again!

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
                let ecs = query_ecs(&mut self.ecs, &system.query)?;

                // Write input data
                let recv_buf = ReceiveBuf {
                    system: Some(system_idx),
                    inbox: std::mem::take(&mut plugin.inbox),
                    ecs,
                };

                // Run plugin
                let ret = plugin.code.dispatch(&recv_buf)?;

                // Write back to ECS
                // TODO: Defer this? It's currently in Arbitrary order!
                apply_ecs_updates(&mut self.ecs, &ret)?;

                // Receive outbox
                plugin.outbox = ret.outbox;
            }
        }

        // Distribute messages
        for i in 0..self.plugins.len() {
            for msg in std::mem::take(&mut self.plugins[i].outbox) {
                if let Some(destinations) = self.indices.get(&msg.channel) {
                    for j in destinations {
                        self.plugins[*j]
                            .inbox
                            .entry(msg.channel)
                            .or_default()
                            .push(msg.clone());
                    }
                } else {
                    eprintln!(
                        "Message on channel {:?} from plugin {} has no destination",
                        msg.channel, i
                    );
                }

                if let Some(inbox) = self.external_inbox.get_mut(&msg.channel) {
                    inbox.push(msg.clone());
                }
            }
        }

        Ok(())
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

    /// Broadcast a message
    pub fn send<M: Message>(&mut self, data: M) {
        let msg = MessageData {
            channel: M::CHANNEL,
            data: serialize(&data).expect("Failed to serialize message"),
        };

        if let Some(indices) = self.indices.get(&M::CHANNEL) {
            for idx in indices {
                self.plugins[*idx]
                    .inbox
                    .entry(M::CHANNEL)
                    .or_default()
                    .push(msg.clone());
            }
        }
    }
}

fn query_ecs(ecs: &mut Ecs, query: &Query) -> Result<EcsData> {
    let entities = ecs.query(query).into_iter().collect();
    let mut components = vec![vec![]; query.len()];

    for &entity in &entities {
        for (term, comp) in query.iter().zip(&mut components) {
            comp.extend_from_slice(ecs.get_raw(entity, term.component));
        }
    }

    Ok(EcsData {
        entities,
        components,
    })
}

fn apply_ecs_updates(ecs: &mut Ecs, send: &SendBuf) -> Result<()> {
    // Apply commands
    for command in &send.commands {
        // TODO: Throw error on modification of non-queried data...
        match command {
            EngineCommand::Create(id) => ecs.import_entity(*id),
            EngineCommand::Delete(id) => ecs.remove_entity(*id),
            EngineCommand::AddComponent(entity, component, data) => {
                ecs.add_component_raw(*entity, *component, data)
            }
        }
    }

    Ok(())
}
