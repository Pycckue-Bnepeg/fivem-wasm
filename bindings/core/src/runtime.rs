extern crate alloc;

use futures::{
    channel::oneshot::{self, Sender},
    executor::{LocalPool, LocalSpawner},
    task::{LocalSpawnExt, SpawnError},
    TryFutureExt,
};

use core::alloc::Layout;
use std::future::Future;
use std::{cell::RefCell, time::Instant};
use std::{collections::BTreeMap, time::Duration};

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
    static TIMERS: RefCell<BTreeMap<Instant, Sender<()>>> = RefCell::new(BTreeMap::new());
}

/// Spawns a new local future that will be polled at next tick or a new event comming
pub fn spawn<Fut: Future<Output = ()> + 'static>(future: Fut) -> Result<(), SpawnError> {
    SPAWNER.with(|sp| sp.borrow().spawn_local(future))
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
        let expiered: Vec<Instant> = timers.range(..=now).map(|(time, _)| time.clone()).collect();

        for key in expiered {
            if let Some(tx) = timers.remove(&key) {
                let _ = tx.send(());
            }
        }
    });
}

/// A stupid timer ...
pub fn sleep_for(duration: Duration) -> impl Future<Output = ()> {
    let instant = Instant::now();
    let (tx, rx) = oneshot::channel();

    TIMERS.with(|timers| {
        let mut timers = timers.borrow_mut();
        timers.insert(instant.checked_add(duration).unwrap(), tx);
    });

    rx.unwrap_or_else(|_| ())
}
