use anyhow::Result;
use cimvr_common::{StringMessage, Transform};
use cimvr_engine::{
    interface::{
        prelude::{query, Access},
        system::Stage,
    },
    Engine,
};
use std::path::PathBuf;

fn main() -> Result<()> {
    let base_path: PathBuf = "target/wasm32-unknown-unknown/release/".into();

    let paths = [
        base_path.join("plugin.wasm").into(),
        //base_path.join("plugin2.wasm").into(),
    ];

    let mut engine = Engine::new(&paths)?;
    engine.init()?;

    engine.subscribe::<StringMessage>();

    engine.send(Stage::Input, StringMessage("Server says haiiiii :3".into()));

    for i in 0..4 {
        println!("ITERATION {i}:");
        engine.dispatch(Stage::Input)?;
        for msg in engine.inbox::<StringMessage>() {
            dbg!(msg);
        }

        for entity in engine.ecs().query(&[query::<Transform>(Access::Read)]) {
            let t = engine.ecs().get::<Transform>(entity);
            println!("{:?}", t);
        }

        let ent = engine.ecs().create_entity();
        engine.ecs().add_component(ent, &Transform::default());
    }

    Ok(())
}
