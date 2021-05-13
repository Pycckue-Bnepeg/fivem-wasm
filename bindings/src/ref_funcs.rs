use crate::types::ScrObject;
use serde::{de::DeserializeOwned, Serialize};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[doc(hidden)]
mod ffi {
    #[link(wasm_import_module = "host")]
    extern "C" {
        pub fn canonicalize_ref(ref_idx: u32, buffer: *mut i8, buffer_size: usize) -> i32;
    }
}

thread_local! {
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(vec![0; 1 << 15]);
    static RETVAL: RefCell<ScrObject> = RefCell::new(ScrObject { data: 0, length: 0 });
    static HANDLERS: RefCell<HashMap<u32, Rc<RefCell<InnerRefFunction>>>> = RefCell::new(HashMap::new());
}

#[no_mangle]
#[doc(hidden)]
// TODO: Add return value
pub unsafe extern "C" fn __cfx_call_ref(
    ref_idx: u32,
    args: *const u8,
    args_len: usize,
) -> *const ScrObject {
    let args = std::slice::from_raw_parts(args, args_len);

    HANDLERS.with(|handlers| {
        let handlers = handlers.borrow();

        if let Some(handler) = handlers.get(&ref_idx) {
            let mut handler = handler.borrow_mut();
            handler.handle(args);
        }
    });

    RETVAL.with(|scr| scr.as_ptr())
}

#[no_mangle]
#[doc(hidden)]
pub unsafe extern "C" fn __cfx_duplicate_ref(ref_idx: u32) -> u32 {
    HANDLERS.with(|handlers| {
        let handlers = handlers.borrow();
        handlers
            .get(&ref_idx)
            .map(|handler| handler.borrow_mut().refs += 1);
    });

    ref_idx
}

#[no_mangle]
#[doc(hidden)]
pub unsafe extern "C" fn __cfx_remove_ref(ref_idx: u32) {
    HANDLERS.with(|handlers| {
        let mut handlers = handlers.borrow_mut();

        if let Some(handler) = handlers.get(&ref_idx) {
            handler.borrow_mut().refs -= 1;

            if handler.borrow().refs <= 0 {
                handlers.remove(&ref_idx);
            }
        }
    });
}

fn canonicalize_ref(ref_idx: u32) -> String {
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

struct InnerRefFunction {
    idx: u32,
    func: Box<dyn FnMut(&[u8]) -> Vec<u8>>,
    refs: i32,
}

impl InnerRefFunction {
    fn handle(&mut self, input: &[u8]) -> Vec<u8> {
        (self.func)(input)
    }
}

#[derive(Clone)]
pub struct RefFunction {
    name: String,
    inner: Rc<RefCell<InnerRefFunction>>,
}

impl RefFunction {
    pub fn new<Handler, Input, Output>(mut handler: Handler) -> RefFunction
    where
        Handler: FnMut(Input) -> Output + 'static,
        Input: DeserializeOwned,
        Output: Serialize,
    {
        thread_local! {
            static REF_IDX: RefCell<u32> = RefCell::new(0);
        }

        let idx = REF_IDX.with(|idx| {
            let mut idx = idx.borrow_mut();
            *idx += 1;
            *idx
        });

        let name = canonicalize_ref(idx);

        let func = move |input: &[u8]| -> Vec<u8> {
            let input = rmp_serde::decode::from_read(input).unwrap();
            let out = handler(input);
            rmp_serde::encode::to_vec(&out).unwrap()
        };

        let inner = InnerRefFunction {
            idx,
            func: Box::new(func),
            refs: 0,
        };

        let inner = Rc::new(RefCell::new(inner));

        HANDLERS.with(|handlers| {
            let mut handlers = handlers.borrow_mut();
            handlers.insert(idx, inner.clone());
        });

        RefFunction {
            name,
            inner: inner.clone(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
