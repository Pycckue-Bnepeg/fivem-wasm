#[no_mangle]
#[doc(hidden)]
pub extern "C" fn __cfx_call_ref(
    ref_idx: u32,
    args: *const u8,
    args_len: usize,
    ret_buf: *mut u8,
    ret_buf_size: usize,
) -> usize {
    0
}
