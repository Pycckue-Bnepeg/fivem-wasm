// internal

extern crate alloc;

use core::alloc::Layout;
use std::ffi::CString;

mod ffi {
    #[link(wasm_import_module = "host")]
    extern "C" {
        pub fn log(ptr: i32, len: i32);
    }
}

fn log<T: AsRef<str>>(message: T) {
    let msg = message.as_ref();
    let cstr = CString::new(msg).unwrap();
    let bytes = cstr.as_bytes_with_nul();

    unsafe {
        ffi::log(bytes.as_ptr() as _, bytes.len() as _);
    }
}

#[no_mangle]
pub unsafe extern "C" fn __alloc(size: u32, align: u32) -> *mut u8 {
    let layout = Layout::from_size_align_unchecked(size as _, align as _);
    alloc::alloc::alloc(layout)
}

#[no_mangle]
pub unsafe extern "C" fn __free(ptr: *mut u8, size: u32, alignment: u32) {
    let layout = Layout::from_size_align_unchecked(size as _, alignment as _);
    alloc::alloc::dealloc(ptr, layout);
}

// internal end

#[no_mangle]
pub unsafe extern "C" fn on_event(
    cstring: *const i8,
    args: *const u8,
    args_length: u32,
    source: *const i8,
) {
    let text = std::ffi::CStr::from_ptr(cstring)
        .to_str()
        .unwrap()
        .to_owned();

    let args = Vec::from(std::slice::from_raw_parts(args, args_length as _));
    let source = std::ffi::CStr::from_ptr(source)
        .to_str()
        .unwrap()
        .to_owned();

    let args = decode_args(&args);

    log(format!("event: {:?}", text));
    log(format!("source: {:?}", source));
    log(format!("args: {:?}", args));
}

fn decode_args(buffer: &[u8]) -> rmpv::Value {
    let mut bf = buffer;
    rmpv::decode::read_value(&mut bf).unwrap()
}

#[no_mangle]
pub extern "C" fn _start() {
    log("I AM FUCKING STARTED !");
}
