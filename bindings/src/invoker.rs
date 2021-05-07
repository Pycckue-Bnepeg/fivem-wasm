#[doc(hidden)]
pub mod ffi {
    #[link(wasm_import_module = "host")]
    extern "C" {
        pub fn invoke(hash_hi: i32, hash_lo: i32, ptr: i32, len: i32);
    }
}

#[derive(Debug)]
pub enum Val<'a> {
    Integer(u64),
    Float(f32),
    String(&'a str),
    Bytes(&'a [u8]),
}

#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pad_0: u32,

    pub y: f32,
    pad_1: u32,

    pub z: f32,
    pad_2: u32,
}

#[repr(C)]
pub struct ScrObject {
    data: *const u8,
    length: usize,
}

impl RetVal for () {
    const IDENT: usize = 0;

    fn convert(_: &[u8]) -> Self {
        ()
    }
}

impl RetVal for String {
    const IDENT: usize = 2;

    fn convert(bytes: &[u8]) -> Self {
        unsafe {
            let cstr = std::ffi::CStr::from_ptr(bytes.as_ptr() as *const _);
            cstr.to_str().unwrap().to_owned()
        }
    }
}

impl RetVal for Vector3 {
    const IDENT: usize = 3;

    fn convert(bytes: &[u8]) -> Self {
        unsafe { std::mem::transmute_copy(&bytes) }
    }
}

pub trait RetVal {
    const IDENT: usize;

    fn convert(bytes: &[u8]) -> Self;
}

pub fn invoke<'a, Ret, Args>(hash: u64, arguments: Args) -> Ret
where
    Ret: RetVal,
    Args: IntoIterator<Item = &'a Val<'a>>,
{
    let iter = arguments.into_iter();

    let hash_hi = (hash >> 32) as i32;
    let hash_lo = (hash & 0xFFFFFFFF) as i32;

    let mut args: Vec<u32> = Vec::new();
    let mut strings = Vec::new(); // cleanup memory after a call

    for arg in iter {
        match arg {
            Val::Integer(int) => {
                args.push(int as *const _ as _);
            }

            Val::Float(float) => {
                args.push(float as *const _ as _);
            }

            Val::String(string) => {
                let cstr = std::ffi::CString::new(*string).unwrap();
                let ptr = cstr.as_bytes_with_nul().as_ptr();

                strings.push(cstr);
                args.push(ptr as _);
            }

            Val::Bytes(bytes) => {
                args.push(bytes.as_ptr() as _);
            }
        }
    }

    unsafe {
        ffi::invoke(hash_hi, hash_lo, args.as_ptr() as _, args.len() as _);
    }

    Ret::convert(&[])
}

/// A FiveM runtime native. Registers current resource as an event handler.
/// Means that if someone triggers an event with this name the resource will be notified.
pub fn register_resource_as_event_handler(event: &str) {
    invoke::<(), _>(0xD233A168, &[Val::String(event)]);
}

/*

    scrObject {
        data: *const u8,
        length: usize,
    }

    scrVector {
        x: f32,
        pad_0: u32,
        y: f32,
        pad_1: u32,
        z: f32,
        pad_2: u32,
    }

*/
