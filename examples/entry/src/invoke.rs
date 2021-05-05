mod ffi {
    #[link(wasm_import_module = "host")]
    extern "C" {
        pub fn invoke(h1: i32, h2: i32, ptr: i32, len: i32);
    }
}

pub fn register_resource_as_event_handler(event: &str) {
    let hash = 0xD233A168u64;

    let h1 = (hash >> 32) as i32;
    let h2 = (hash & 0xFFFFFFFF) as i32;

    let cstr = std::ffi::CString::new(event).unwrap();
    let args = vec![cstr.as_ptr() as i32];

    unsafe {
        ffi::invoke(h1, h2, args.as_ptr() as _, args.len() as _);
    }
}
