#[doc(hidden)]
pub mod ffi {
    #[link(wasm_import_module = "host")]
    extern "C" {
        pub fn invoke(hash_hi: i32, hash_lw: i32, ptr: i32, len: i32);
    }
}

pub fn invoke<'a, T: IntoIterator<Item = &'a *const i8>>(hash: u64, arguments: T) {
    let args: Vec<*const i8> = arguments.into_iter().map(|ptr| *ptr).collect();

    let h1 = (hash >> 32) as i32;
    let h2 = (hash & 0xFFFFFFFF) as i32;

    unsafe {
        ffi::invoke(h1, h2, args.as_ptr() as _, args.len() as _);
    }
}

pub fn register_resource_as_event_handler(event: &str) {
    let cstr = std::ffi::CString::new(event).unwrap();
    invoke(0xD233A168, &[cstr.as_ptr()]);
}

// TODO: calling conv
// #[macro_export]
// macro_rules! invoke_native {
//     ($hash:expr) => {{
//         let hash = $hash as u32 as u64;
//         let h1 = (hash >> 32) as i32;
//         let h2 = (hash & 0xFFFFFFFF) as i32;

//         $crate::invoker::ffi::invoke(h1, h2, 0, 0);
//     }};

//     ($hash:expr, $($args:tt)+) => {{
//         let hash = $hash as u32 as u64;
//         let h1 = (hash >> 32) as i32;
//         let h2 = (hash & 0xFFFFFFFF) as i32;

//         let mut args = Vec::with_capacity(32);

//         $crate::invoker::invoke_native!(@ args, $($args)+);

//         $crate::invoker::ffi::invoke(h1, h2, args.as_ptr() as _, args.len() as _);
//     }};

//     (@ $cont:ident, $arg:expr, $($rest:tt)+) => {
//         $crate::invoker::invoke_native!(@ $cont, $arg);
//         $crate::invoker::invoke_native!(@ $cont, $($rest)+);
//     };

//     (@ $cont:ident, $arg:expr) => {

//     };
// }
