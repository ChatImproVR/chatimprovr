pub mod ecs;
pub mod plugin;
use std::path::PathBuf;

use anyhow::Result;
pub use cimvr_engine_interface as interface;
use ecs::Ecs;
use interface::serial::{EcsData, ReceiveBuf, SendBuf};
use plugin::Plugin;

pub struct Engine {
    wt: wasmtime::Engine,
    plugins: Vec<Plugin>,
    ecs: Ecs,
}

impl Engine {
    pub fn new(plugins: &[PathBuf]) -> Result<Self> {
        let wt = wasmtime::Engine::new(&Default::default())?;
        let plugins: Vec<Plugin> = plugins
            .iter()
            .map(|p| Plugin::new(&wt, p))
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
            let ret = plugin.dispatch(&recv_buf)?;
            apply_ecs_updates(&mut self.ecs, &ret)?;
        }

        Ok(())
    }

    pub fn ecs(&mut self) -> &mut Ecs {
        &mut self.ecs
    }
}

fn apply_ecs_updates(ecs: &mut Ecs, send: &SendBuf) -> Result<()> {
    Ok(())
}
