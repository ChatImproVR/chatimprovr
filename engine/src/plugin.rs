use anyhow::{Context, Result};
use cimvr_engine_interface::serial::{
    deserialize, serialize_into, serialized_size, ReceiveBuf, SendBuf,
};
use rand::prelude::*;
use std::{io::Cursor, path::Path};
use wasmtime::{Caller, Extern, Func, Instance, Memory, Module, Store, TypedFunc};

#[allow(dead_code)]
pub struct Plugin {
    store: Store<()>,
    module: Module,
    instance: Instance,
    mem: Memory,
    print_fn: Func,
    random_fn: Func,
    dispatch_fn: TypedFunc<(), u32>,
    reserve_fn: TypedFunc<u32, u32>,
}

impl Plugin {
    /// Load the plugin in an uninitialized state
    pub fn new(wt: &wasmtime::Engine, plugin_path: impl AsRef<Path>) -> Result<Self> {
        let bytes = std::fs::read(plugin_path)?;
        let module = Module::new(wt, &bytes)?;
        let mut store = Store::new(wt, ());

        // Basic printing functionality
        let print_fn = Func::wrap(
            &mut store,
            |mut caller: Caller<'_, ()>, ptr: u32, len: u32| {
                // TODO: Shorten this
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                let mut buf = vec![0; len as usize];
                mem.read(caller, ptr as usize, &mut buf).unwrap();
                let s = String::from_utf8(buf).unwrap();
                print!("{}", s);
            },
        );

        // Random number "syscall". TODO: Include this in SendBuf instead?
        let random_fn = Func::wrap(&mut store, || rand::thread_rng().gen::<u64>());

        let mut imports: Vec<Extern> = vec![];
        let warn = "Did you try to use an IO function from outside the sandbox like File::open()?\n\
                    You may only use io functions supplied by the host! Maybe you want include_bytes!()";

        for imp in module.imports() {
            match (imp.name(), imp.ty()) {
                ("_print", wasmtime::ExternType::Func(_)) => {
                    imports.push(print_fn.into());
                }
                ("_random", wasmtime::ExternType::Func(_)) => {
                    imports.push(random_fn.into());
                }
                (other, wasmtime::ExternType::Func(ty)) => {
                    log::info!("Faking {other}");
                    let func = Func::new(&mut store, ty, |_, _, _| Ok(()));
                    imports.push(func.into());
                }
                _ => panic!("{:?}", imp), //log::warn!("{}\nUnhandled import {:#?}", warn, imp),
                                          //_ => todo!(),
            }
        }

        let instance = Instance::new(&mut store, &module, &imports)?;

        let mem = instance.get_memory(&mut store, "memory").unwrap();

        let dispatch_fn = instance
            .get_typed_func::<(), u32>(&mut store, "_dispatch")
            .context("Getting dispatch function. Did you forget to use make_app_state!()? RTFM!")?;
        let reserve_fn = instance.get_typed_func::<u32, u32>(&mut store, "_reserve")?;

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

    /// Dispatch plugin internals with given intent
    pub fn dispatch(&mut self, recv: &ReceiveBuf) -> Result<SendBuf> {
        // Rerve needed space within the plugin's memory
        let size = serialized_size(&recv)?;
        let ptr = self
            .reserve_fn
            .call(&mut self.store, size as u32)
            .context("Reserve")?;

        // Serialize directly into the module's memory. Saves time!
        let mem = self.mem.data_mut(&mut self.store);
        let cursor = Cursor::new(&mut mem[ptr as usize..][..size]);
        serialize_into(cursor, &recv).expect("Serializing plugin input. This is a bug!");

        // Call the plugin
        let ptr = self
            .dispatch_fn
            .call(&mut self.store, ())
            .context("Dispatch")?;

        // Also deserialize directly from the module's memory
        let mem = self.mem.data_mut(&mut self.store);
        let ptr = ptr as usize;

        // Read header for length
        let mut header_bytes = [0; 4];
        let (header, forever_after) = mem[ptr..].split_at(header_bytes.len());
        header_bytes.copy_from_slice(&header);
        let payload_len = u32::from_le_bytes(header_bytes) as usize;

        let slice = &forever_after[..payload_len];

        // Deserialize it
        Ok(deserialize(Cursor::new(slice)).context("Deserializing bincode")?)
    }
}
