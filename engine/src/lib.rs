pub mod ecs;
pub mod plugin;
use std::{collections::HashMap, path::PathBuf};

use anyhow::{Ok, Result};
pub use cimvr_engine_interface as interface;
use ecs::Ecs;
use interface::{
    prelude::*,
    serial::{EcsData, ReceiveBuf, SendBuf},
    system::Stage,
};
use plugin::Plugin;

// Keep the ECS in an Arc, so that it may be read simultaneously

/// Plugin management structure
struct PluginState {
    /// Plugin code and interface
    code: Plugin,
    /// Systems on this plugin
    systems: Vec<SystemDescriptor>,
    /// Message inbox
    inbox: HashMap<ChannelId, Vec<Message>>,
}

/// Plugin state, plugin code, ECS state, messaging machinery, and more
pub struct Engine {
    _wasm: wasmtime::Engine,
    plugins: Vec<PluginState>,
    ecs: Ecs,
}

impl PluginState {
    pub fn new(code: Plugin) -> Self {
        PluginState {
            code,
            systems: vec![],
            inbox: HashMap::default(),
        }
    }
}

impl Engine {
    pub fn new(plugins: &[PathBuf]) -> Result<Self> {
        let wasm = wasmtime::Engine::new(&Default::default())?;
        let plugins: Vec<PluginState> = plugins
            .iter()
            .map(|p| Plugin::new(&wasm, p))
            .map(|plugin| plugin.map(PluginState::new))
            .collect::<Result<_>>()?;
        let ecs = Ecs::new();

        Ok(Self {
            _wasm: wasm,
            plugins,
            ecs,
        })
    }

    /// Initialize plugin code. Must be called at least once!
    /// This is seperate from the constructor so that you may differentiate between loading errors
    /// and init errors, and also to allow you to decide when plugin code actually begins executing.
    pub fn init(&mut self) -> Result<()> {
        for plugin in &mut self.plugins {
            let send = ReceiveBuf {
                system: None,
                ecs: EcsData::default(),
            };
            let recv = plugin.code.dispatch(&send)?;

            apply_ecs_updates(&mut self.ecs, &recv)?;

            plugin.systems = recv.sched;
        }

        Ok(())
    }

    /// Dispatch plugin code on the given stage
    pub fn dispatch(&mut self, stage: Stage) -> Result<()> {
        for plugin in &mut self.plugins {
            for (system_idx, system) in plugin.systems.iter().enumerate() {
                // Filter to the requested stage
                if system.stage != stage {
                    continue;
                }

                let ecs = query_ecs(&mut self.ecs, &system.query)?;

                // TODO: Prep ECS data here!
                let recv_buf = ReceiveBuf {
                    system: Some(system_idx),
                    ecs,
                };

                let ret = plugin.code.dispatch(&recv_buf)?;

                apply_ecs_updates(&mut self.ecs, &ret)?;
            }
        }

        Ok(())
    }

    pub fn ecs(&mut self) -> &mut Ecs {
        &mut self.ecs
    }
}

fn query_ecs(ecs: &Ecs, query: &Query) -> Result<EcsData> {
    let entities = ecs.query(query).into_iter().collect();
    let mut components = vec![vec![]; query.len()];

    for &entity in &entities {
        for (term, comp) in query.iter().zip(&mut components) {
            comp.extend_from_slice(ecs.get(entity, term.component));
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
                ecs.add_component(*entity, *component, data)
            }
        }
    }

    Ok(())
}
