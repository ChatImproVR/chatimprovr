use std::path::Path;
use anyhow::Result;
use wasmtime::{Module, Instance, Store, Func, Caller};

pub struct Engine {
    wt: wasmtime::Engine,
    module: Module,
    store: Store<()>,
    instance: Instance,
}

impl Engine {
    pub fn new(plugin_path: impl AsRef<Path>) -> Result<Self> {
        let wt = wasmtime::Engine::new(&Default::default())?;
        let bytes = std::fs::read(plugin_path)?;
        let module = Module::new(&wt, &bytes)?;
        let mut store = Store::new(&wt, ());

        let print_fn = Func::wrap(&mut store, |ptr: u32, len: u32| {
            println!("PINT {} {}", ptr, len);
        });

        let instance = Instance::new(&mut store, &module, &[print_fn.into()])?;

        Ok(Self {
            wt,
            module,
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