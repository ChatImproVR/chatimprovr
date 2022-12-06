use anyhow::Result;

fn main() -> Result<()> {
    let path = "target/wasm32-unknown-unknown/release/plugin.wasm";
    let mut engine = cimvr_engine::Engine::new(path)?;
    engine.dispatch()?;

    Ok(())
}
