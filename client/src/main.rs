use anyhow::Result;
use cimvr_engine::{interface::system::Stage, Engine};
use std::path::PathBuf;

fn main() -> Result<()> {
    let path: PathBuf = "target/wasm32-unknown-unknown/release/plugin.wasm".into();

    let mut engine = Engine::new(&[path.into()])?;
    engine.init()?;

    for _ in 0..10 {
        engine.dispatch(Stage::Input)?;
    }

    Ok(())
}
