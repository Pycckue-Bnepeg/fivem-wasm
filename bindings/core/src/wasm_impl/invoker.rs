use crate::invoker::RETVAL_BUFFER;

#[no_mangle]
pub extern "C" fn __cfx_extend_retval_buffer(new_size: usize) -> *const u8 {
    RETVAL_BUFFER.with(|retval| {
        let mut vec = retval.borrow_mut();
        vec.resize(new_size, 0);

        vec.as_ptr()
    })
}
