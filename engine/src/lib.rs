use anyhow::{bail, Result};
use cimvr_engine_interface::serial::{
    serialize_into, serialized_size, EcsData, EngineIntrinsics, ReceiveBuf, SendBuf,
};
use rand::prelude::*;
use std::path::Path;
use wasmtime::{
    AsContextMut, Caller, Extern, Func, ImportType, Instance, Memory, Module, Store, TypedFunc,
};

pub mod ecs;

pub struct Engine {
    wt: wasmtime::Engine,
    module: Module,
    print_fn: Func,
    random_fn: Func,
    mem: Memory,
    store: Store<()>,
    instance: Instance,
    dispatch_fn: TypedFunc<(), u32>,
    reserve_fn: TypedFunc<u32, u32>,
}

impl Engine {
    pub fn new(plugin_path: impl AsRef<Path>) -> Result<Self> {
        let wt = wasmtime::Engine::new(&Default::default())?;
        let bytes = std::fs::read(plugin_path)?;
        let module = Module::new(&wt, &bytes)?;
        let mut store = Store::new(&wt, ());

        // Basic printing functionality
        let print_fn = Func::wrap(
            &mut store,
            |mut caller: Caller<'_, ()>, ptr: u32, len: u32| {
                // TODO: What a disaster
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                let mut buf = vec![0; len as usize];
                mem.read(caller, ptr as usize, &mut buf).unwrap();
                let s = String::from_utf8(buf).unwrap();
                print!("{}", s);
            },
        );

        // Basic printing functionality
        let random_fn = Func::wrap(&mut store, || rand::thread_rng().gen::<u64>());

        let mut imports: Vec<Extern> = vec![];
        for imp in module.imports() {
            match (imp.name(), imp.ty()) {
                ("_print", wasmtime::ExternType::Func(_)) => {
                    imports.push(print_fn.into());
                }
                ("_random", wasmtime::ExternType::Func(_)) => {
                    imports.push(random_fn.into());
                }
                _ => bail!("Unhandled import {:#?}", imp),
            }
        }

        let instance = Instance::new(&mut store, &module, &imports)?;

        let mem = instance.get_memory(&mut store, "memory").unwrap();

        let dispatch_fn = instance.get_typed_func::<(), u32, _>(&mut store, "_dispatch")?;
        let reserve_fn = instance.get_typed_func::<u32, u32, _>(&mut store, "_reserve")?;

        Ok(Self {
            random_fn,
            mem,
            wt,
            module,
            print_fn,
            store,
            instance,
            dispatch_fn,
            reserve_fn,
        })
    }

    pub fn dispatch(&mut self) -> Result<()> {
        let recv_buf = ReceiveBuf {
            system: 0,
            ecs: EcsData {
                entities: vec![],
                components: vec![],
            },
            messages: vec![],
            intrinsics: EngineIntrinsics { random: 0 },
        };

        // Serialize directly into the module's memory. Saves time!
        let size = serialized_size(&recv_buf)?;
        let ptr = self.reserve_fn.call(&mut self.store, size as u32)?;
        let mem = self.mem.data_mut(&mut self.store);
        let slice = std::io::Cursor::new(&mut mem[ptr as usize..][..size]);
        serialize_into(slice, &recv_buf)?;

        // Call the plugin!
        self.dispatch_fn.call(&mut self.store, ())?;

        // Also deserialize directly from the module's memory
        // Read length header that the plugin provides

        Ok(())
    }
}
