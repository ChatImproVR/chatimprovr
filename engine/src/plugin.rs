use anyhow::{bail, Result};
use cimvr_engine_interface::serial::{
    deserialize, serialize_into, serialized_size, EcsData, ReceiveBuf, SendBuf,
};
use rand::prelude::*;
use std::{io::Cursor, path::Path};
use wasmtime::{
    AsContextMut, Caller, Extern, Func, ImportType, Instance, Memory, Module, Store, TypedFunc,
};

pub struct Plugin {
    module: Module,
    print_fn: Func,
    random_fn: Func,
    mem: Memory,
    store: Store<()>,
    instance: Instance,
    dispatch_fn: TypedFunc<(), u32>,
    reserve_fn: TypedFunc<u32, u32>,
}

impl Plugin {
    pub fn new(wt: &wasmtime::Engine, plugin_path: impl AsRef<Path>) -> Result<Self> {
        let bytes = std::fs::read(plugin_path)?;
        let module = Module::new(wt, &bytes)?;
        let mut store = Store::new(wt, ());

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
            module,
            print_fn,
            store,
            instance,
            dispatch_fn,
            reserve_fn,
        })
    }

    pub fn dispatch(&mut self, recv: &ReceiveBuf) -> Result<SendBuf> {
        // Serialize directly into the module's memory. Saves time!
        let size = serialized_size(&recv)?;
        let ptr = self.reserve_fn.call(&mut self.store, size as u32)?;
        let mem = self.mem.data_mut(&mut self.store);
        let cursor = Cursor::new(&mut mem[ptr as usize..][..size]);
        serialize_into(cursor, &recv)?;

        // Call the plugin!
        let ptr = self.dispatch_fn.call(&mut self.store, ())?;

        // Also deserialize directly from the module's memory
        // Read length header that the plugin provides
        let mem = self.mem.data_mut(&mut self.store);
        let ptr = ptr as usize;

        let (header, xs) = mem[ptr..].split_at(4);

        // Read header for length
        let mut header_bytes = [0; 4];
        header_bytes.copy_from_slice(&header);
        let len = u32::from_le_bytes(header_bytes) as usize;
        let slice = &xs[..len];

        // Deserialize it
        Ok(deserialize(Cursor::new(slice))?)
    }
}
