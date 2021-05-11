use crate::types::{RetVal, ReturnValue};

const RETVAL_BUFFER_SIZE: usize = 1 << 15;

thread_local! {
    static RETVAL_BUFFER: Vec<u8> = Vec::with_capacity(RETVAL_BUFFER_SIZE);
}

#[doc(hidden)]
pub mod ffi {
    #[link(wasm_import_module = "host")]
    extern "C" {
        pub fn invoke(hash_hi: i32, hash_lo: i32, ptr: i32, len: i32, retval: i32) -> i32;
    }
}

#[derive(Debug)]
pub enum Val<'a> {
    Integer(u64),
    Float(f32),
    String(&'a str),
    Bytes(&'a [u8]),
}

#[derive(Debug, Clone)]
pub enum InvokeError {
    NoSpace,
}

// TODO: Result<Ret, ()>
pub fn invoke<'a, Ret, Args>(hash: u64, arguments: Args) -> Result<Ret, InvokeError>
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

    RETVAL_BUFFER.with(|buf| unsafe {
        let retval = ReturnValue::new::<Ret>(buf);

        let ret_len = ffi::invoke(
            hash_hi,
            hash_lo,
            args.as_ptr() as _,
            args.len() as _,
            (&retval) as *const _ as i32,
        );

        if ret_len == -1 {
            return Err(InvokeError::NoSpace);
        }

        let read_buf = std::slice::from_raw_parts(buf.as_ptr(), ret_len as usize);

        Ok(Ret::convert(read_buf))
    })
}

/// A FiveM runtime native. Registers current resource as an event handler.
/// Means that if someone triggers an event with this name the resource will be notified.
pub fn register_resource_as_event_handler(event: &str) {
    let _ = invoke::<(), _>(0xD233A168, &[Val::String(event)]);
}
