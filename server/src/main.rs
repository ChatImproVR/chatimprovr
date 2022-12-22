use anyhow::Result;
use cimvr_engine::{interface::system::Stage, Engine};
use std::path::PathBuf;

fn main() -> Result<()> {
    // Set up logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse args
    let args = std::env::args().skip(1);
    let paths: Vec<PathBuf> = args.map(PathBuf::from).collect();

    // Set up engine and initialize plugins
    let mut engine = Engine::new(&paths, true)?;

    engine.init_plugins()?;
    engine.dispatch(Stage::Input)?;

    Ok(())
}
