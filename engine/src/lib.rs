use anyhow::Result;
use cimvr_engine_interface::serial::{
    serialize, serialized_size, EngineIntrinsics, ReceiveBuf, SendBuf,
};
use std::path::Path;
use wasmtime::{AsContextMut, Caller, Func, Instance, Memory, Module, Store, TypedFunc};

pub struct Engine {
    wt: wasmtime::Engine,
    module: Module,
    print_fn: Func,
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

        let instance = Instance::new(&mut store, &module, &[print_fn.into()])?;

        let mem = instance.get_memory(&mut store, "memory").unwrap();

        let dispatch_fn = instance.get_typed_func::<(), u32, _>(&mut store, "_dispatch")?;
        let reserve_fn = instance.get_typed_func::<u32, u32, _>(&mut store, "_reserve")?;

        Ok(Self {
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
            entities: vec![],
            components: vec![],
            messages: vec![],
            intrinsics: EngineIntrinsics { random: 0 },
        };

        // Serialize directly into the module's memory. Saves time!
        let size = serialized_size(&recv_buf)?;
        let ptr = self.reserve_fn.call(&mut self.store, size as u32)?;
        let mem = self.mem.data_mut(&mut self.store);
        let slice = std::io::Cursor::new(&mut mem[ptr as usize..][..size]);
        serialize(slice, &recv_buf)?;

        // Call the plugin!
        self.dispatch_fn.call(&mut self.store, ())?;

        // Also deserialize directly from the module's memory
        // Read length header that the plugin provides

        Ok(())
    }
}
