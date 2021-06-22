use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ref_funcs::InnerRefFunction;
use crate::types::*;

thread_local! {
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(vec![0; 1 << 15]);
    static RETVAL: RefCell<ScrObject> = RefCell::new(ScrObject { data: 0, length: 0 });
    pub(crate) static HANDLERS: RefCell<HashMap<u32, Rc<InnerRefFunction>>> = RefCell::new(HashMap::new());
    pub(crate) static REF_IDX: RefCell<u32> = RefCell::new(0);
}

mod ffi {
    #[link(wasm_import_module = "host")]
    extern "C" {
        pub fn canonicalize_ref(ref_idx: u32, buffer: *mut i8, buffer_size: usize) -> i32;
    }
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn __cfx_call_ref(
    ref_idx: u32,
    args: *const u8,
    args_len: usize,
) -> *const ScrObject {
    let args = std::slice::from_raw_parts(args, args_len);

    HANDLERS.with(|handlers| {
        let handlers = handlers.borrow();

        if let Some(handler) = handlers.get(&ref_idx) {
            BUFFER.with(|buf| {
                handler.handle(args, buf);

                RETVAL.with(|retval| {
                    let mut retval = retval.borrow_mut();
                    let buf = buf.borrow();

                    retval.data = buf.as_ptr() as _;
                    retval.length = buf.len() as _;
                });
            });
        }
    });

    RETVAL.with(|scr| scr.as_ptr())
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn __cfx_duplicate_ref(ref_idx: u32) -> u32 {
    HANDLERS.with(|handlers| {
        let handlers = handlers.borrow();
        if let Some(handler) = handlers.get(&ref_idx) {
            let refs = handler.refs.get();
            handler.refs.set(refs + 1);
        }
    });

    ref_idx
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn __cfx_remove_ref(ref_idx: u32) {
    HANDLERS.with(|handlers| {
        let remove = {
            let handlers = handlers.borrow();

            if let Some(handler) = handlers.get(&ref_idx) {
                let refs = handler.refs.get();
                handler.refs.set(refs - 1);

                refs <= 1
            } else {
                false
            }
        };

        if remove {
            handlers.borrow_mut().remove(&ref_idx);
        }
    });
}

pub(crate) fn canonicalize_ref(ref_idx: u32) -> String {
    thread_local! {
        static CANON_REF: RefCell<Vec<u8>> = RefCell::new(vec![0; 1024]);
    }

    CANON_REF.with(|vec| {
        let mut resized = false;

        loop {
            let (buffer, buffer_size) = {
                let mut vec = vec.borrow_mut();

                (vec.as_mut_ptr() as *mut _, vec.capacity())
            };

            let result = unsafe { ffi::canonicalize_ref(ref_idx, buffer, buffer_size) };

            if result == 0 {
                // some error?
                return String::new();
            }

            if result < 0 {
                if resized {
                    return String::new();
                }

                vec.borrow_mut().resize(result.abs() as _, 0);
                resized = true;
            } else {
                unsafe {
                    let slice = std::slice::from_raw_parts(buffer as *mut u8, (result - 1) as _);

                    return std::str::from_utf8_unchecked(slice).to_owned();
                }
            }
        }
    })
}
