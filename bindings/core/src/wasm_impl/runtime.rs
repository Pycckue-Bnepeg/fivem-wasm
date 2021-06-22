extern crate alloc;

use futures::{
    channel::oneshot::Sender,
    executor::{LocalPool, LocalSpawner},
};

use core::alloc::Layout;
use std::collections::BTreeMap;
use std::{cell::RefCell, time::Instant};

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
    pub(crate) static SPAWNER: RefCell<LocalSpawner> = LOCAL_POOL.with(|lp| RefCell::new(lp.borrow().spawner()));
    pub(crate) static TIMERS: RefCell<BTreeMap<Instant, Vec<Sender<()>>>> = RefCell::new(BTreeMap::new());
}

#[no_mangle]
pub extern "C" fn __cfx_on_tick() {
    fire_timers();

    LOCAL_POOL.with(|lp| {
        let mut lp = lp.borrow_mut();
        lp.run_until_stalled();
    });
}

fn fire_timers() {
    let now = Instant::now();

    TIMERS.with(|timers| {
        let mut timers = timers.borrow_mut();
        let expiered: Vec<Instant> = timers.range(..=now).map(|(time, _)| *time).collect();

        for key in expiered {
            if let Some(senders) = timers.remove(&key) {
                for tx in senders {
                    let _ = tx.send(());
                }
            }
        }
    });
}
