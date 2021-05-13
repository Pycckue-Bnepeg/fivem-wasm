extern crate alloc;

use core::alloc::Layout;

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn __cfx_alloc(size: u32, align: u32) -> *mut u8 {
    let layout = Layout::from_size_align_unchecked(size as _, align as _);
    alloc::alloc::alloc(layout)
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn __cfx_free(ptr: *mut u8, size: u32, align: u32) {
    let layout = Layout::from_size_align_unchecked(size as _, align as _);
    alloc::alloc::dealloc(ptr, layout);
}
