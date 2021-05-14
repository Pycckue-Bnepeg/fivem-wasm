use crate::{
    ref_funcs::RefFunction,
    types::{GuestArg, RetVal, ReturnValue, Vector3},
};

use std::cell::RefCell;

const RETVAL_BUFFER_SIZE: usize = 1 << 15;

thread_local! {
    static RETVAL_BUFFER: RefCell<Vec<u8>> = RefCell::new(vec![0; RETVAL_BUFFER_SIZE]);
}

#[doc(hidden)]
pub mod ffi {
    #[link(wasm_import_module = "host")]
    extern "C" {
        pub fn invoke(
            hash_hi: i32,
            hash_lo: i32,
            ptr: *const crate::types::GuestArg,
            len: usize,
            retval: *const crate::types::ReturnValue,
        ) -> i32;

        // pub fn invoke_ref_func() -> i32;
    }
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn __cfx_extend_retval_buffer(new_size: usize) -> *const u8 {
    crate::log(format!("request to resize with new size: {}", new_size));

    RETVAL_BUFFER.with(|retval| {
        let mut vec = retval.borrow_mut();
        vec.resize(new_size, 0);

        vec.as_ptr()
    })
}

// #[derive(Debug)]
// TODO: Add refs
pub enum Val<'a> {
    RefInteger(&'a u32),
    RefFloat(&'a f32),
    RefLong(&'a u64),

    MutRefInteger(&'a mut u32),
    MutRefFloat(&'a mut f32),
    MutRefLong(&'a mut u64),

    Integer(u32),
    Float(f32),
    Long(u64),

    Vector3(Vector3),
    RefVector3(&'a Vector3),
    MutRefVector3(&'a mut Vector3),

    String(&'a str),
    Bytes(&'a [u8]),
    MutBytes(&'a mut [u8]),

    RefFunc(RefFunction),
}

#[derive(Debug, Clone)]
pub enum InvokeError {
    NullResult,
    NoSpace,
    Code(i32),
}

// TODO Iterator::map
pub fn invoke<'a, Ret, Args>(hash: u64, arguments: Args) -> Result<Ret, InvokeError>
where
    Ret: RetVal,
    Args: IntoIterator<Item = &'a Val<'a>>,
{
    let iter = arguments.into_iter();

    let hash_hi = (hash >> 32) as i32;
    let hash_lo = (hash & 0xFFFFFFFF) as i32;

    let mut args: Vec<GuestArg> = Vec::new();
    let mut strings = Vec::new(); // cleanup memory after a call

    for arg in iter {
        match arg {
            Val::Integer(int) => {
                args.push(GuestArg::new(int, false));
            }

            Val::Float(float) => {
                args.push(GuestArg::new(float, false));
            }

            Val::Long(long) => {
                args.push(GuestArg::new(long, false));
            }

            Val::RefInteger(int) => {
                args.push(GuestArg::new(int, true));
            }

            Val::RefFloat(float) => {
                args.push(GuestArg::new(float, true));
            }

            Val::RefLong(long) => {
                args.push(GuestArg::new(long, true));
            }

            Val::MutRefInteger(int) => {
                args.push(GuestArg::new(int, true));
            }

            Val::MutRefFloat(float) => {
                args.push(GuestArg::new(float, true));
            }

            Val::MutRefLong(long) => {
                args.push(GuestArg::new(long, true));
            }

            Val::Vector3(vec) => {
                args.push(GuestArg::new(vec, false));
            }

            Val::RefVector3(vec) => {
                args.push(GuestArg::new(vec, true));
            }

            Val::MutRefVector3(vec) => {
                args.push(GuestArg::new(vec, true));
            }

            Val::String(string) => {
                let cstr = std::ffi::CString::new(*string).unwrap();
                let ptr = cstr.as_bytes_with_nul().as_ptr();

                strings.push(cstr);

                args.push(GuestArg::new(unsafe { &*ptr }, true));
            }

            Val::Bytes(bytes) => {
                args.push(GuestArg::new(bytes, true));
            }

            Val::MutBytes(bytes) => {
                args.push(GuestArg::new(bytes, true));
            }

            Val::RefFunc(func) => {
                let cstr = std::ffi::CString::new(func.name()).unwrap();
                let ptr = cstr.as_bytes_with_nul().as_ptr();

                strings.push(cstr);

                args.push(GuestArg::new(unsafe { &*ptr }, true));
            }
        }
    }

    RETVAL_BUFFER.with(|buf| unsafe {
        let retval = ReturnValue::new::<Ret>(&buf.borrow());

        let ret_len = ffi::invoke(
            hash_hi,
            hash_lo,
            args.as_ptr(),
            args.len(),
            (&retval) as *const _,
        );

        if ret_len == -4 {
            return Err(InvokeError::NullResult);
        }

        if ret_len == -1 {
            return Err(InvokeError::NoSpace);
        }

        if ret_len < 0 {
            return Err(InvokeError::Code(ret_len));
        }

        let read_buf = std::slice::from_raw_parts(buf.borrow().as_ptr(), ret_len as usize);

        Ok(Ret::convert(read_buf))
    })
}

/// A FiveM runtime native. Registers current resource as an event handler.
/// Means that if someone triggers an event with this name the resource will be notified.
pub fn register_resource_as_event_handler(event: &str) -> Result<(), InvokeError> {
    invoke(0xD233A168, &[Val::String(event)])
}
