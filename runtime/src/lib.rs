use std::ffi::CStr;

use fivem::types::{ReturnType, ReturnValue, ScrObject, Vector3};
use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, Wasi};

// mod walloc;

pub type LogFunc = extern "C" fn(msg: *const i8);
pub type InvokeFunc = extern "C" fn(args: *mut NativeContext) -> u32;
pub type CanonicalizeRefFunc =
    extern "C" fn(ref_idx: u32, buffer: *mut i8, buffer_size: u32) -> i32;

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
static mut CANONICALIZE_REF: Option<CanonicalizeRefFunc> = None;

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

    // TODO: call on_tick
    pub fn tick(&mut self) {}

    pub fn call_ref(&mut self, ref_idx: u32, args: &[u8], ret_buf: &mut Vec<u8>) -> u32 {
        self.script
            .as_ref()
            .and_then(|script| call_call_ref(script, ref_idx, args, ret_buf))
            .unwrap_or_default()
    }

    pub fn duplicate_ref(&mut self, ref_idx: u32) -> u32 {
        self.script
            .as_ref()
            .and_then(|script| {
                let func = script
                    .instance
                    .get_typed_func::<i32, i32>("__cfx_duplicate_ref")
                    .ok()?;

                func.call(ref_idx as _).map(|idx| idx as _).ok()
            })
            .unwrap_or_default()
    }

    pub fn remove_ref(&mut self, ref_idx: u32) {
        self.script.as_ref().and_then(|script| {
            let func = script
                .instance
                .get_typed_func::<i32, ()>("__cfx_remove_ref")
                .ok()?;

            func.call(ref_idx as _).ok()
        });
    }

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
        let on_event = instance.get_func("__cfx_on_event");

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
                |caller: Caller, h1: i32, h2: i32, ptr: i32, len: i32, retval: i32| -> i32 {
                    // array ptr, array len
                    let mut buf = vec![0u32; len as usize];
                    let mem = caller.get_export("memory").unwrap().into_memory().unwrap();

                    if len > 0 && ptr != 0 {
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

                    let retval = if retval == 0 {
                        None
                    } else {
                        Some(unsafe {
                            let ptr = mem.data_ptr().add(retval as _) as *const ReturnValue;
                            &*ptr
                        })
                    };

                    let resize_func = caller
                        .get_export("__cfx_extend_retval_buffer")
                        .and_then(|export| export.into_func());

                    let call_result = call_native(hash, buf.as_slice(), mem, retval, resize_func);

                    match call_result {
                        CallResult::NoReturn => -2,
                        CallResult::NoSpace => -1,
                        CallResult::OkWithLen(len) => len as _,
                        CallResult::Ok => 0,
                    }
                },
            )
            .unwrap();

        linker
            .func(
                "host",
                "canonicalize_ref",
                |caller: Caller, ref_idx: i32, ptr: i32, len: i32| {
                    let mem = caller.get_export("memory").unwrap().into_memory().unwrap();

                    unsafe {
                        let ptr = mem.data_ptr().add(ptr as _) as *mut _;

                        if let Some(canonicalize_ref) = CANONICALIZE_REF {
                            return canonicalize_ref(ref_idx as _, ptr, len as _);
                        }
                    }

                    return 0;
                },
            )
            .unwrap();

        let module = Module::new(engine, bytes).unwrap();
        let instance = linker.instantiate(&module).unwrap();

        let start = instance.get_func("_start").expect("no _start entry");
        let on_event = instance.get_func("__cfx_on_event");

        start.call(&[]).unwrap();

        ScriptModule {
            store,
            instance,
            on_event,
        }
    }

    fn alloc_bytes(&self, bytes: &[u8]) -> (u32, usize) {
        let malloc = self
            .instance
            .get_typed_func::<(i32, u32), u32>("__cfx_alloc")
            .unwrap();

        let data_ptr = malloc.call((bytes.len() as _, 1)).unwrap();
        let mem = self.instance.get_memory("memory").unwrap();

        mem.write(data_ptr as _, bytes).unwrap();

        return (data_ptr, bytes.len());
    }

    fn free_bytes(&self, (offset, length): (u32, usize)) {
        let free = self
            .instance
            .get_typed_func::<(u32, u32, u32), ()>("__cfx_free")
            .unwrap();

        free.call((offset as _, length as _, 1)).unwrap();
    }
}

pub fn set_logger(log: LogFunc) {
    unsafe {
        LOGGER = Some(log);
    }
}

pub fn set_native_invoke(invoke: extern "C" fn(ctx: *mut std::ffi::c_void) -> u32) {
    unsafe {
        INVOKE = Some(std::mem::transmute(invoke));
    }
}

pub fn set_canonicalize_ref(canonicalize_ref: CanonicalizeRefFunc) {
    unsafe {
        CANONICALIZE_REF = Some(canonicalize_ref);
    }
}

enum CallResult {
    NoReturn,
    NoSpace,
    OkWithLen(u32),
    Ok,
}

fn call_native(
    hash: u64,
    args: &[u32],
    memory: Memory,
    retval: Option<&ReturnValue>,
    resize_func: Option<Func>,
) -> CallResult {
    let mut ctx = NativeContext::default();

    ctx.native_identifier = hash;
    ctx.num_arguments = args.len() as _;

    let mem_start = unsafe { memory.data_unchecked().as_ptr() };

    for (idx, &offset) in args.iter().enumerate() {
        unsafe {
            ctx.arguments[idx] = mem_start.offset(offset as isize) as usize;
        };
    }

    if let Some(invoke) = unsafe { INVOKE } {
        invoke(&mut ctx);
    }

    if ctx.num_results == 0 || retval.is_none() {
        return CallResult::Ok;
    }

    if let Some(retval) = retval {
        let resize_buffer = |new_size: usize| -> Option<*mut u8> {
            if let Some(resizer) = resize_func.as_ref() {
                let ptr = resizer.call(&[Val::I32(new_size as _)]).ok()?;

                ptr.get(0).and_then(|val| val.i32()).and_then(|ptr| {
                    if ptr == 0 {
                        None
                    } else {
                        Some(unsafe { memory.data_ptr().add(ptr as _) })
                    }
                })
            } else {
                None
            }
        };

        let mut buffer = unsafe { memory.data_ptr().add(retval.buffer as _) };
        let rettype = ReturnType::from(retval.rettype as u32);

        match rettype {
            ReturnType::Empty => CallResult::Ok,
            ReturnType::Number => {
                if retval.capacity < 4 {
                    if let Some(new_buffer) = resize_buffer(4) {
                        buffer = new_buffer
                    } else {
                        return CallResult::NoSpace;
                    }
                }

                unsafe {
                    *(buffer as *mut u32) = ctx.arguments[0] as u32;
                }

                return CallResult::OkWithLen(4);
            }

            ReturnType::String => {
                let cstr = unsafe { CStr::from_ptr(ctx.arguments[0] as *const _) };
                let bytes = cstr.to_bytes();
                let len = bytes.len();

                if retval.capacity < len as _ {
                    if let Some(new_buffer) = resize_buffer(len) {
                        buffer = new_buffer
                    } else {
                        return CallResult::NoSpace;
                    }
                }

                unsafe {
                    std::ptr::copy(bytes.as_ptr(), buffer, len);
                }

                CallResult::OkWithLen(len as _)
            }

            ReturnType::Vector3 => {
                let vec = ctx.arguments.as_ptr() as *const Vector3;
                let len = std::mem::size_of::<Vector3>();

                if retval.capacity < len as _ {
                    if let Some(new_buffer) = resize_buffer(len) {
                        buffer = new_buffer
                    } else {
                        return CallResult::NoSpace;
                    }
                }

                unsafe {
                    std::ptr::copy(vec, buffer as *mut _, 1);
                }

                return CallResult::OkWithLen(len as _);
            }

            ReturnType::MsgPack => {
                let scrobj =
                    unsafe { &*(ctx.arguments.as_ptr() as *const fivem::types::ScrObject) };

                let len = scrobj.length;

                if (retval.capacity as u64) < len {
                    if let Some(new_buffer) = resize_buffer(len as usize) {
                        buffer = new_buffer
                    } else {
                        return CallResult::NoSpace;
                    }
                }

                unsafe {
                    std::ptr::copy(scrobj.data as *const u8, buffer, len as _);
                }

                return CallResult::OkWithLen(len as _);
            }

            ReturnType::Unk => CallResult::NoReturn,
        };
    }

    CallResult::NoReturn
}

fn call_call_ref(
    script: &ScriptModule,
    ref_idx: u32,
    args: &[u8],
    ret_buf: &mut Vec<u8>,
) -> Option<u32> {
    let memory = script.instance.get_memory("memory")?;
    let cfx_call_ref = script
        .instance
        .get_typed_func::<(i32, i32, i32), i32>("__cfx_call_ref")
        .ok()?;

    let args_guest = script.alloc_bytes(args);

    let scrobj = {
        let result = cfx_call_ref
            .call((ref_idx as _, args_guest.0 as _, args.len() as _))
            .ok();

        script.free_bytes(args_guest);

        result?
    };

    if scrobj == 0 {
        return None;
    }

    let scrobj = unsafe {
        let ptr = memory.data_ptr().add(scrobj as _) as *const ScrObject;
        &*ptr
    };

    unsafe {
        ret_buf.set_len(0);
    }

    if scrobj.data == 0 || scrobj.length == 0 {
        return None;
    }

    let slice = unsafe {
        let ptr = memory.data_ptr().add(scrobj.data as _);
        std::slice::from_raw_parts(ptr, scrobj.length as _)
    };

    ret_buf.extend_from_slice(slice);

    // if scrobj.length > ret_buf.capacity() as _ {
    //     ret_buf.resize(scrobj.length as usize, 0);
    // }

    Some(ret_buf.len() as _)
}
