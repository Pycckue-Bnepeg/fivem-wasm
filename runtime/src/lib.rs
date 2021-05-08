use std::ffi::CStr;

use fivem::types::ReturnTypes;
use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, Wasi};

pub type LogFunc = extern "C" fn(message: *const i8);
pub type InvokeFunc = extern "C" fn(ctx: *mut NativeContext);

#[repr(C)]
#[derive(Default)]
pub struct NativeContext {
    arguments: [usize; 32],
    num_arguments: u32,
    num_results: u32,
    native_identifier: u64,
}

static mut LOGGER: Option<LogFunc> = None;
static mut INVOKE: Option<InvokeFunc> = None;

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
                let ev = script.alloc_bytes(event_name.to_bytes_with_nul());
                let args = script.alloc_bytes(args);
                let src = script.alloc_bytes(source.to_bytes_with_nul());

                // event, args, args_len, src
                let _ = func.call(&[
                    Val::I32(ev.0 as _),
                    Val::I32(args.0 as _),
                    Val::I32(args.1 as _),
                    Val::I32(src.0 as _),
                ]);

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

        linker
            .func(
                "host",
                "invoke",
                |caller: Caller, h1: i32, h2: i32, ptr: i32, len: i32, rettype: i32| -> i32 {
                    // array ptr, array len
                    let mut buf = vec![0u32; len as usize];
                    let mem = caller.get_export("memory").unwrap().into_memory().unwrap();

                    if len > 0 {
                        let tmp = unsafe {
                            let ptr = buf.as_mut_ptr() as *mut u8;
                            std::slice::from_raw_parts_mut(
                                ptr,
                                len as usize * std::mem::size_of::<i32>(),
                            )
                        };

                        mem.read(ptr as _, tmp).unwrap();
                    }

                    let h1 = h1 as u32;
                    let h2 = h2 as u32;
                    let hash = ((h1 as u64) << 32) + (h2 as u64);
                    let mut ctx = NativeContext::default();

                    ctx.native_identifier = hash;
                    ctx.num_arguments = len as _;

                    for (idx, &offset) in buf.iter().enumerate() {
                        unsafe {
                            let mem_start = mem.data_unchecked().as_ptr();
                            ctx.arguments[idx] = mem_start.offset(offset as isize) as usize;
                        };
                    }

                    if let Some(invoke) = unsafe { INVOKE } {
                        invoke(&mut ctx);
                    }

                    if ctx.num_results == 0 {
                        return 0;
                    }

                    let rettype = ReturnTypes::from(rettype as u32);

                    match rettype {
                        ReturnTypes::Empty => 0,
                        ReturnTypes::Number => ctx.arguments[0] as _,
                        ReturnTypes::String => ctx.arguments[0] as _,

                        ReturnTypes::Vector3 => {
                            let vec = ctx.arguments.as_ptr() as *const fivem::types::Vector3;

                            Self::alloc_value(&caller, unsafe { &*vec }).0 as _
                        }

                        ReturnTypes::MsgPack => {
                            let scrobj = ctx.arguments.as_ptr() as *const fivem::types::ScrObject;

                            let scrobj = unsafe { &*scrobj };
                            let bytes = unsafe {
                                std::slice::from_raw_parts(
                                    scrobj.data as *const u8,
                                    scrobj.length as _,
                                )
                            };

                            let (ptr, _) = Self::alloc_vec(&caller, bytes);

                            let scrobj = fivem::types::ScrObject {
                                data: ptr as _,
                                length: scrobj.length,
                            };

                            Self::alloc_value(&caller, &scrobj).0 as _
                        }

                        ReturnTypes::Unk => 0,
                    }
                },
            )
            .unwrap();

        let module = Module::new(engine, bytes).unwrap();
        let instance = linker.instantiate(&module).unwrap();

        let start = instance.get_func("_start").expect("no _start entry");
        let on_event = instance.get_func("on_event");

        start.call(&[]).unwrap();

        ScriptModule {
            store,
            instance,
            on_event,
        }
    }

    // TODO: Make an allocation module?
    fn alloc_value<T: Sized>(caller: &Caller, value: &T) -> (u32, std::alloc::Layout) {
        let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
        let malloc = caller.get_export("__alloc").unwrap().into_func().unwrap();
        let malloc = malloc.typed::<(i32, u32), u32>().unwrap();

        let layout = std::alloc::Layout::new::<T>();

        let data_ptr = malloc
            .call((layout.size() as _, layout.align() as _))
            .unwrap();

        unsafe {
            let mem = mem.data_ptr().add(data_ptr as _) as *mut T;
            std::ptr::copy(value, mem, 1);
        }

        return (data_ptr, layout);
    }

    fn alloc_vec<T: Sized>(caller: &Caller, value: &[T]) -> (u32, std::alloc::Layout) {
        let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
        let malloc = caller.get_export("__alloc").unwrap().into_func().unwrap();
        let malloc = malloc.typed::<(i32, u32), u32>().unwrap();

        let layout = std::alloc::Layout::array::<T>(value.len()).unwrap();

        let data_ptr = malloc
            .call((layout.size() as _, layout.align() as _))
            .unwrap();

        unsafe {
            let mem = mem.data_ptr().add(data_ptr as _) as *mut T;
            std::ptr::copy(value.as_ptr(), mem, value.len());
        }

        return (data_ptr, layout);
    }

    fn alloc_bytes(&self, bytes: &[u8]) -> (u32, usize) {
        let malloc = self
            .instance
            .get_typed_func::<(i32, u32), u32>("__alloc")
            .unwrap();

        let data_ptr = malloc.call((bytes.len() as _, 1)).unwrap();
        let mem = self.instance.get_memory("memory").unwrap();

        mem.write(data_ptr as _, bytes).unwrap();

        return (data_ptr, bytes.len());
    }

    fn free_bytes(&self, (offset, length): (u32, usize)) {
        let free = self
            .instance
            .get_typed_func::<(u32, u32, u32), ()>("__free")
            .unwrap();

        free.call((offset as _, length as _, 1)).unwrap();
    }
}

pub fn set_logger(log: LogFunc) {
    unsafe {
        LOGGER = Some(log);
    }
}

pub fn set_native_invoke(invoke: extern "C" fn(ctx: *mut std::ffi::c_void)) {
    unsafe {
        INVOKE = Some(std::mem::transmute(invoke));
    }
}
