use anyhow::Result;

fn main() -> Result<()> {
    let mut engine = cimvr_engine::Engine::new("target/wasm32-unknown-unknown/debug/plugin.wasm")?;
    engine.dispatch()?;

    Ok(())
}
