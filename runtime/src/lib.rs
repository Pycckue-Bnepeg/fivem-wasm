use std::ffi::CStr;

use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, Wasi};

pub type LogFunc = extern "C" fn(message: *const i8);

static mut LOGGER: Option<LogFunc> = None;

pub struct Runtime {
    engine: Engine,
    script: Option<ScriptModule>,
}

impl Runtime {
    pub fn new() -> Runtime {
        let engine = Engine::default();

        Runtime {
            engine,
            script: None,
        }
    }

    pub fn load_module(&mut self, bytes: &[u8], is_server: bool) {
        let script = if is_server {
            ScriptModule::new_with_wasi(&self.engine, bytes)
        } else {
            ScriptModule::new(&self.engine, bytes)
        };

        self.script = Some(script);
    }

    pub fn trigger_event(&mut self, event_name: &CStr, args: &[u8], source: &CStr) {
        if let Some(script) = self.script.as_mut() {
            if let Some(func) = &script.on_event {
                println!("RUNTIME: {:?}", event_name);
                println!("RUNTIME: {:?}", event_name.to_bytes_with_nul());
                println!("RUNTIME: {:?}", event_name.to_str().unwrap());

                let ev = script.alloc_bytes(event_name.to_bytes_with_nul());
                let args = script.alloc_bytes(args);
                let src = script.alloc_bytes(source.to_bytes_with_nul());

                // event, args, args_len, src
                let res = func.call(&[
                    Val::I32(ev.0 as _),
                    Val::I32(args.0 as _),
                    Val::I32(args.1 as _),
                    Val::I32(src.0 as _),
                ]);

                println!("call result: {:?}", res);

                script.free_bytes(ev);
                script.free_bytes(args);
                script.free_bytes(src);
            }
        }
    }

    pub fn tick(&mut self) {}

    pub fn memory_size(&self) -> u32 {
        self.script
            .as_ref()
            .and_then(|script| script.instance.get_memory("memory"))
            .map(|memory| memory.size())
            .unwrap_or(0)
    }
}

struct ScriptModule {
    store: Store,
    instance: Instance,
    on_event: Option<Func>,
}

impl ScriptModule {
    fn new(engine: &Engine, bytes: &[u8]) -> ScriptModule {
        let store = Store::new(&engine);
        let module = Module::new(engine, bytes).unwrap();

        let instance = Instance::new(&store, &module, &[]).unwrap();
        let on_event = instance.get_func("on_event");

        ScriptModule {
            store,
            instance,
            on_event,
        }
    }

    fn new_with_wasi(engine: &Engine, bytes: &[u8]) -> ScriptModule {
        let store = Store::new(&engine);
        let mut linker = Linker::new(&store);

        let wasi = Wasi::new(
            &store,
            WasiCtxBuilder::new()
                .inherit_stdout()
                .inherit_stdio()
                .inherit_stderr()
                .build()
                .unwrap(),
        );

        wasi.add_to_linker(&mut linker).unwrap();

        linker
            .func("host", "log", |caller: Caller, ptr: i32, len: i32| {
                let mut buf = vec![0u8; len as usize];
                let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
                mem.read(ptr as _, buf.as_mut()).unwrap();

                unsafe {
                    if let Some(logger) = LOGGER {
                        logger(buf.as_mut_ptr() as _);
                    }
                }
            })
            .unwrap();

        let module = Module::new(engine, bytes).unwrap();
        let instance = linker.instantiate(&module).unwrap();

        let on_event = instance.get_func("on_event");

        ScriptModule {
            store,
            instance,
            on_event,
        }
    }

    fn alloc_bytes(&self, bytes: &[u8]) -> (u32, usize) {
        let malloc = self
            .instance
            .get_typed_func::<(i32, u32), u32>("__alloc")
            .unwrap();

        let data_ptr = malloc.call((bytes.len() as _, 0)).unwrap();
        let mem = self.instance.get_memory("memory").unwrap();

        mem.write(data_ptr as _, bytes).unwrap();

        return (data_ptr, bytes.len());
    }

    fn free_bytes(&self, (offset, length): (u32, usize)) {
        let free = self
            .instance
            .get_typed_func::<(u32, u32, u32), ()>("__free")
            .unwrap();

        free.call((offset as _, length as _, 0)).unwrap();
    }
}

pub fn set_logger(log: LogFunc) {
    unsafe {
        LOGGER = Some(log);
    }
}
