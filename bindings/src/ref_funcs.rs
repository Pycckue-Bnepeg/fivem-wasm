use std::cell::RefCell;

use crate::types::ScrObject;

thread_local! {
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(vec![0; 1 << 15]);
    static RETVAL: RefCell<ScrObject> = RefCell::new(ScrObject { data: 0, length: 0 });
}

#[no_mangle]
#[doc(hidden)]
pub unsafe extern "C" fn __cfx_call_ref(
    ref_idx: u32,
    args: *const u8,
    args_len: usize,
) -> *const ScrObject {
    RETVAL.with(|scr| scr.as_ptr())
}
