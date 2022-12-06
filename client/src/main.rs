use anyhow::Result;
use cimvr_engine::cimvr_engine_interface::serial::{EcsData, EngineIntrinsics, ReceiveBuf};

fn main() -> Result<()> {
    let path = "target/wasm32-unknown-unknown/release/plugin.wasm";

    let recv_buf = ReceiveBuf {
        system: 0,
        ecs: EcsData {
            entities: vec![],
            components: vec![],
        },
        messages: vec![],
        intrinsics: EngineIntrinsics { random: 0 },
    };

    let mut engine = cimvr_engine::Plugin::new(path)?;
    let ret = engine.dispatch(&recv_buf)?;

    dbg!(ret);

    Ok(())
}
