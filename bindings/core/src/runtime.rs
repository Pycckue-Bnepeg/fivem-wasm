extern crate alloc;

use futures::{
    executor::{LocalPool, LocalSpawner},
    task::{LocalSpawnExt, SpawnError},
};

use core::alloc::Layout;
use std::cell::RefCell;
use std::future::Future;

#[no_mangle]
pub unsafe extern "C" fn __cfx_alloc(size: u32, align: u32) -> *mut u8 {
    let layout = Layout::from_size_align_unchecked(size as _, align as _);
    alloc::alloc::alloc(layout)
}

#[no_mangle]
pub unsafe extern "C" fn __cfx_free(ptr: *mut u8, size: u32, align: u32) {
    let layout = Layout::from_size_align_unchecked(size as _, align as _);
    alloc::alloc::dealloc(ptr, layout);
}

thread_local! {
    pub(crate) static LOCAL_POOL: RefCell<LocalPool> = RefCell::new(LocalPool::new());
    static SPAWNER: RefCell<LocalSpawner> = LOCAL_POOL.with(|lp| RefCell::new(lp.borrow().spawner()));
}

/// Spawns a new local future that will be polled at next tick or a new event comming
pub fn spawn<Fut: Future<Output = ()> + 'static>(future: Fut) -> Result<(), SpawnError> {
    SPAWNER.with(|sp| sp.borrow().spawn_local(future))
}

#[no_mangle]
pub extern "C" fn __cfx_on_tick() {
    LOCAL_POOL.with(|lp| {
        let mut lp = lp.borrow_mut();
        lp.run_until_stalled();
    });
}
