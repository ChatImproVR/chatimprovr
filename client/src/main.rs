use std::{path::PathBuf, time::Instant};

use anyhow::Result;
use cimvr_engine::{
    interface::serial::{EcsData, ReceiveBuf},
    plugin::Plugin,
    Engine,
};

fn main() -> Result<()> {
    let path: PathBuf = "target/wasm32-unknown-unknown/release/plugin.wasm".into();

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

    let t = Instant::now();
    let n = 500_000;
    for _ in 0..n {
        let recv = plugin.dispatch(&ReceiveBuf {
            system: Some(0),
            ecs: EcsData {
                entities: vec![],
                components: vec![],
            },
            messages: vec![],
        })?;
    }

    let t = t.elapsed();
    println!("{}s", t.as_secs_f32());
    println!("{} FPS", n as f32 / t.as_secs_f32());

    Ok(())
}
