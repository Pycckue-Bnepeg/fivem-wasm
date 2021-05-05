use std::ffi::{c_void, CStr};

pub type LogFunc = extern "C" fn(msg: *const i8);

#[no_mangle]
pub extern "C" fn wasm_create_runtime() -> *mut c_void {
    let runtime = cfx_wasm_runtime::Runtime::new();
    let boxed = Box::new(runtime);

    Box::into_raw(boxed) as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn wasm_destroy_runtime(runtime: *mut c_void) {
    let runtime = runtime as *mut cfx_wasm_runtime::Runtime;

    Box::from_raw(runtime);
}

#[no_mangle]
pub unsafe extern "C" fn wasm_runtime_create_module(
    runtime: *mut c_void,
    bytes: *const u8,
    length: u64,
) {
    let runtime = &mut *(runtime as *mut cfx_wasm_runtime::Runtime);
    let bytes = std::slice::from_raw_parts(bytes, length as usize);

    runtime.load_module(bytes, true);
}

#[no_mangle]
pub unsafe extern "C" fn wasm_runtime_destroy_module(runtime: *mut c_void) {}

#[no_mangle]
pub unsafe extern "C" fn wasm_runtime_tick(runtime: *mut c_void) {
    let runtime = &mut *(runtime as *mut cfx_wasm_runtime::Runtime);
    runtime.tick();
}

#[no_mangle]
pub unsafe extern "C" fn wasm_runtime_trigger_event(
    runtime: *mut c_void,
    event_name: *const i8,
    args: *const u8,
    args_len: u32,
    source: *const i8,
) {
    let runtime = &mut *(runtime as *mut cfx_wasm_runtime::Runtime);

    let event = CStr::from_ptr(event_name);
    let args = std::slice::from_raw_parts(args, args_len as _);
    let source = CStr::from_ptr(source);

    runtime.trigger_event(event, args, source);
}

#[no_mangle]
pub unsafe extern "C" fn wasm_set_logger_function(log: LogFunc) {
    cfx_wasm_runtime::set_logger(log);
}

#[no_mangle]
pub unsafe extern "C" fn wasm_runtime_memory_usage(runtime: *mut c_void) -> u32 {
    let runtime = &mut *(runtime as *mut cfx_wasm_runtime::Runtime);

    runtime.memory_size() * 64 * 1024
}
