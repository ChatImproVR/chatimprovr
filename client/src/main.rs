use std::path::PathBuf;

use anyhow::Result;
use cimvr_engine::{
    interface::serial::{EcsData, ReceiveBuf},
    plugin::Plugin,
    Engine,
};

fn main() -> Result<()> {
    let path: PathBuf = "target/wasm32-unknown-unknown/debug/plugin.wasm".into();

    let wasm = Engine::new(&Default::default())?;
    let mut plugin = Plugin::new(&wasm, &path)?;
    let recv = plugin.dispatch(&ReceiveBuf {
        system: None,
        ecs: EcsData {
            entities: vec![],
            components: vec![],
        },
        messages: vec![],
    })?;

    dbg!(recv);

    Ok(())
}
