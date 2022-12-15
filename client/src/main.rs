use anyhow::Result;
use cimvr_engine::{interface::system::Stage, Engine};
use std::path::PathBuf;

fn main() -> Result<()> {
    let base_path: PathBuf = "target/wasm32-unknown-unknown/release/".into();

    let paths = [
        base_path.join("plugin.wasm").into(),
        base_path.join("plugin2.wasm").into(),
    ];

    let mut engine = Engine::new(&paths)?;
    engine.init()?;

    for i in 0..4 {
        println!("ITERATION {i}:");
        engine.dispatch(Stage::Input)?;
    }

    Ok(())
}
