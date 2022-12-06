use std::path::Path;
use anyhow::Result;
use wasmtime::{Module, Instance, Store, Func, Caller, AsContextMut};

pub struct Engine {
    wt: wasmtime::Engine,
    module: Module,
    print_fn: Func,
    store: Store<()>,
    instance: Instance,
}

impl Engine {
    pub fn new(plugin_path: impl AsRef<Path>) -> Result<Self> {
        let wt = wasmtime::Engine::new(&Default::default())?;
        let bytes = std::fs::read(plugin_path)?;
        let module = Module::new(&wt, &bytes)?;
        let mut store = Store::new(&wt, ());

        let print_fn = Func::wrap(&mut store, |mut caller: Caller<'_, ()>, ptr: u32, len: u32| {
            // TODO: What a disaster
            let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
            let mut buf = vec![0; len as usize];
            mem.read(caller, ptr as usize, &mut buf).unwrap();
            let s = String::from_utf8(buf).unwrap();
            print!("{}", s);
        });

        let instance = Instance::new(&mut store, &module, &[print_fn.into()])?;

        Ok(Self {
            wt,
            module,
            print_fn,
            store,
            instance,
        })
    }

    pub fn dispatch(&mut self) -> Result<()> {
        let dispatch_fn = self.instance.get_typed_func::<(), u32, _>(&mut self.store, "_dispatch")?;

        dispatch_fn.call(&mut self.store, ())?;

        Ok(())
    }
}