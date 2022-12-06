pub mod ecs;
pub mod plugin;
use std::path::PathBuf;

use anyhow::Result;
pub use cimvr_engine_interface as interface;
use ecs::Ecs;
use interface::{
    prelude::{Access, EngineCommand, QueryTerm},
    serial::{EcsData, ReceiveBuf, SendBuf, SystemDescriptor},
};
use plugin::Plugin;

struct PluginState {
    code: Plugin,
    systems: Vec<SystemDescriptor>,
}

pub struct Engine {
    wt: wasmtime::Engine,
    plugins: Vec<PluginState>,
    ecs: Ecs,
}

impl Engine {
    pub fn new(plugins: &[PathBuf]) -> Result<Self> {
        let wt = wasmtime::Engine::new(&Default::default())?;
        let plugins: Vec<PluginState> = plugins
            .iter()
            .map(|p| Plugin::new(&wt, p))
            .map(|plugin| {
                Ok(PluginState {
                    code: plugin?,
                    systems: vec![],
                })
            })
            .collect::<Result<_>>()?;
        let ecs = Ecs::new();

        Ok(Self { wt, plugins, ecs })
    }

    pub fn dispatch(&mut self) -> Result<()> {
        let recv_buf = ReceiveBuf {
            system: 0,
            ecs: EcsData {
                entities: vec![],
                components: vec![],
            },
            messages: vec![],
        };

        for plugin in &mut self.plugins {
            let ret = plugin.code.dispatch(&recv_buf)?;

            if plugin.systems.is_empty() {
                plugin.systems = ret.sched;
            }

            apply_ecs_updates(&mut self.ecs, &ret, plugin.systems)?;
        }

        Ok(())
    }

    pub fn ecs(&mut self) -> &mut Ecs {
        &mut self.ecs
    }
}

fn apply_ecs_updates(ecs: &mut Ecs, send: &SendBuf, query: &[QueryTerm]) -> Result<()> {
    // TODO: Apply updates!
    for (comp_idx, term) in query.iter().enumerate() {
        if term.access == Access::Read {
            continue;
        }

        for entity in &send.ecs.entities {
            ecs.get_mut(*entity, term.component)
                .copy_from_slice(&send.ecs.components[comp_idx]);
        }
    }

    // Apply commands
    for command in &send.commands {
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
