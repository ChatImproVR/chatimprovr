use std::path::PathBuf;

use anyhow::Result;
use cimvr_engine::{
    interface::serial::{EcsData, ReceiveBuf},
    Engine,
};

fn main() -> Result<()> {
    let path: PathBuf = "target/wasm32-unknown-unknown/release/plugin.wasm".into();

    let mut engine = Engine::new(&[path])?;
    engine.dispatch()?;

    Ok(())
}
