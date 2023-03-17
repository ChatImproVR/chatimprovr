pub mod ecs;
pub mod hotload;
pub mod network;
pub mod plugin;
pub mod timing;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use timing::Timing;

use anyhow::{format_err, Context, Ok, Result};
pub use cimvr_engine_interface as interface;
use ecs::{apply_ecs_commands, query_ecs_data, Ecs};
use interface::{
    pkg_namespace,
    prelude::*,
    serial::{deserialize, serialize, EcsData, ReceiveBuf},
    system::Stage,
    Saved,
};
use plugin::Plugin;

// Keep the ECS in an Arc, so that it may be read simultaneously
pub struct Config {
    /// Run server-side plugins
    pub is_server: bool,
}

/// Plugin state, plugin code, ECS state, messaging machinery, and more
pub struct Engine {
    /// WASM engine
    wasm: wasmtime::Engine,
    /// Plugin states
    plugins: Vec<PluginState>,
    /// Entity and Component data
    ecs: Ecs,
    /// Message distribution indices, maps (channel id) -> (plugin index, system index)
    indices: HashMap<ChannelId, Vec<(PluginIndex, usize)>>,
    /// Host inboxes
    external_inbox: Inbox,
    /// Network inbox; messages to be sent from plugins to the remote(s)
    network_inbox: Vec<MessageData>,
    /// Configuration we were constructed with
    cfg: Config,
    /// Manages FrameTime
    time: Timing,
}

/// Plugin management structure
struct PluginState {
    /// Path to this plugin's source code
    path: PathBuf,
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

/// Marker of plugin ownership, by plugin index
#[derive(Component, Copy, Clone, Debug, Default, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginIndex(usize);

impl PluginState {
    pub fn new(path: PathBuf, wasm: &wasmtime::Engine) -> Result<Self> {
        let code = Plugin::new(wasm, &path)?;
        Ok(PluginState {
            path,
            code,
            outbox: vec![],
            systems: vec![],
            inbox: Default::default(),
        })
    }

    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }
}

impl Engine {
    /// Load plugins at the given paths
    pub fn new(plugins: &[PathBuf], cfg: Config) -> Result<Self> {
        let time = Timing::init();

        let wasm = wasmtime::Engine::new(&Default::default())?;

        let plugins: Vec<PluginState> = plugins
            .iter()
            .map(|p| {
                PluginState::new(p.clone(), &wasm)
                    .with_context(|| format_err!("Initializing plugin {}", p.display()))
            })
            .collect::<Result<_>>()?;

        let ecs = Ecs::new();

        Ok(Self {
            time,
            wasm,
            indices: HashMap::new(),
            plugins,
            ecs,
            external_inbox: HashMap::new(),
            network_inbox: vec![],
            cfg,
        })
    }

    /// Initialize plugin code. Must be called at least once!
    /// This is seperate from the constructor so that you may differentiate between loading errors
    /// and init errors, and also to allow you to decide when plugin code actually begins executing.
    pub fn init_plugins(&mut self) -> Result<()> {
        // Dispatch all plugins
        for plugin_idx in 0..self.plugins.len() {
            let name = self.plugins[plugin_idx].name();
            self.init_plugin(plugin_idx)
                .with_context(|| format_err!("Plugin {}", name))?;
        }

        // Distribute messages
        self.propagate();

        // Run PostInit stage
        self.dispatch(Stage::PostInit)
            .context("Running PostInit stage")?;

        Ok(())
    }

    fn init_plugin(&mut self, plugin_idx: usize) -> Result<()> {
        log::info!("Initializing {}", self.plugins[plugin_idx].name());
        // Dispatch init signal
        let send = ReceiveBuf {
            system: None,
            inbox: Default::default(),
            ecs: EcsData::default(),
            is_server: self.cfg.is_server,
        };
        let recv = self.plugins[plugin_idx].code.dispatch(&send)?;

        // Apply ECS commands
        apply_ecs_commands(&mut self.ecs, &recv.commands, PluginIndex(plugin_idx))?;

        // Setup message indices for each system
        for (sys_idx, sys) in recv.systems.iter().enumerate() {
            // Set up lookup table
            for channel in &sys.subscriptions {
                self.indices
                    .entry(channel.clone())
                    .or_default()
                    .push((PluginIndex(plugin_idx), sys_idx));
            }

            // Initialize system's inbox
            self.plugins[plugin_idx].inbox = vec![HashMap::new(); recv.systems.len()];
        }

        // Set up schedule, send first messages
        self.plugins[plugin_idx].systems = recv.systems;
        self.plugins[plugin_idx].outbox = recv.outbox;

        Ok(())
    }

    /// Dispatch plugin code on the given stage
    pub fn dispatch(&mut self, stage: Stage) -> Result<()> {
        // TODO: Should this be the responsibility of something else?
        // Pre-update formally marks the start of a new frame
        if stage == Stage::PreUpdate {
            self.time.frame();
        }
        // Send time each frame
        self.send(self.time.get_frame_time());

        // Run plugins
        for i in 0..self.plugins.len() {
            self.dispatch_plugin(stage, i)?;
        }

        // Distribute messages
        self.propagate();

        Ok(())
    }

    pub fn dispatch_plugin(&mut self, stage: Stage, plugin_idx: usize) -> Result<()> {
        let plugin = &mut self.plugins[plugin_idx];
        for (system_idx, system) in plugin.systems.iter().enumerate() {
            // Filter to the requested stage
            if system.stage != stage {
                continue;
            }

            // Query ECS
            let ecs_data = query_ecs_data(&mut self.ecs, &system.query).context("ECS query")?;

            // Write input data
            let recv_buf = ReceiveBuf {
                system: Some(system_idx),
                inbox: std::mem::take(&mut plugin.inbox[system_idx]),
                is_server: self.cfg.is_server,
                ecs: ecs_data,
            };

            // Run plugin
            let name = plugin.name();
            let ret = plugin
                .code
                .dispatch(&recv_buf)
                .with_context(|| format_err!("Running plugin {}", name))?;

            // Write back to ECS
            // TODO: Defer this? It's currently in Arbitrary order!
            apply_ecs_commands(&mut self.ecs, &ret.commands, PluginIndex(plugin_idx))
                .context("Updating ECS after dispatch")?;

            // Receive outbox
            plugin.outbox.extend(ret.outbox);
        }

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

    /// Broadcast the message locally, without checkint to see if it's marked with local locality
    pub fn broadcast_local(&mut self, msg: MessageData) {
        if let Some(destinations) = self.indices.get(&msg.channel) {
            for (PluginIndex(plugin_idx), system_idx) in destinations {
                self.plugins[*plugin_idx].inbox[*system_idx]
                    .entry(msg.channel.clone())
                    .or_default()
                    .push(msg.clone());
            }
        } else {
            log::trace!("Message on channel {:?} has no destination", msg.channel);
        }

        if let Some(inbox) = self.external_inbox.get_mut(&msg.channel) {
            inbox.push(msg.clone());
        }
    }

    /// Access ECS data
    pub fn ecs(&mut self) -> &mut Ecs {
        &mut self.ecs
    }

    /// Subscribe to the given channel
    pub fn subscribe<M: Message>(&mut self) {
        self.external_inbox.entry(M::CHANNEL.into()).or_default();
    }

    /// Drain messages from the given channel (external inbox)
    pub fn inbox<M: Message>(&mut self) -> impl Iterator<Item = M> + '_ {
        self.external_inbox
            .get_mut(&M::CHANNEL.into())
            .expect("Attempted to access a channel we haven't subscribed to")
            .drain(..)
            .map(|msg| {
                deserialize(std::io::Cursor::new(msg.data)).expect("Failed to decode message")
            })
    }

    /// Drain all outgoing network messages
    pub fn network_inbox(&mut self) -> Vec<MessageData> {
        std::mem::take(&mut self.network_inbox)
    }

    /// Broadcast a local message
    pub fn send<M: Message>(&mut self, data: M) {
        self.broadcast(MessageData {
            channel: M::CHANNEL.into(),
            data: serialize(&data).expect("Failed to serialize message"),
            client: None,
        });
    }

    /// Reload the plugin at the given path
    pub fn reload(&mut self, path: PathBuf) -> Result<()> {
        // Find old plugin
        let i = self
            .plugins
            .iter_mut()
            .position(|p| p.path.canonicalize().unwrap() == path.canonicalize().unwrap())
            .expect("Requested plugin is not loaded");

        // Replace old plugin
        let new_plugin = PluginState::new(path.clone(), &self.wasm)?;
        let name = new_plugin.name();

        self.plugins[i] = new_plugin;

        // Delete all unsaved entities from that plugin
        let indices = self
            .ecs
            .query(&[QueryComponent::new::<PluginIndex>(Access::Read)]);
        for ent in indices {
            if let Some(PluginIndex(idx)) = self.ecs().get::<PluginIndex>(ent) {
                // Only those from the plugin that have no Saved component
                if idx == i && self.ecs().get::<Saved>(ent).is_none() {
                    // TODO: This is slow lol
                    self.ecs.remove_entity(ent);
                }
            }
        }

        // Delete message indices for that plugin
        for channel in self.indices.values_mut() {
            channel.retain(|(PluginIndex(j), _)| *j != i);
        }

        // Initialize new plugin
        self.init_plugin(i)
            .with_context(|| format_err!("Initializing reloaded plugin {}", name))?;

        // Propagate startup messages
        self.propagate();

        // Run PostInit stage
        self.dispatch_plugin(Stage::PostInit, i)
    }
}
