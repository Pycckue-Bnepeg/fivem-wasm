pub mod events;
#[doc(hidden)]
pub mod exports;
pub mod invoker;
pub mod ref_funcs;
pub mod runtime;
pub mod types {
    pub use cfx_wasm_rt_types::*;

    pub trait ToMessagePack {
        fn to_message_pack(&self) -> Vec<u8>;
    }

    impl<T: serde::Serialize> ToMessagePack for T {
        fn to_message_pack(&self) -> Vec<u8> {
            rmp_serde::to_vec(self).unwrap_or_else(|_| vec![])
        }
    }
}

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
