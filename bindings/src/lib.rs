pub mod events;
#[doc(hidden)]
pub mod exports;
pub mod invoker;
pub mod runtime;

mod ffi {
    #[link(wasm_import_module = "host")]
    extern "C" {
        pub fn log(ptr: i32, len: i32);
    }
}

/// Logs a message to the FiveM server or client
pub fn log<T: AsRef<str>>(message: T) {
    let msg = message.as_ref();
    let cstr = std::ffi::CString::new(msg).unwrap();
    let bytes = cstr.as_bytes_with_nul();

    unsafe {
        ffi::log(bytes.as_ptr() as _, bytes.len() as _);
    }
}
