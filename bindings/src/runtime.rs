use futures::{
    executor::LocalPool,
    task::{LocalSpawnExt, SpawnError},
};
use std::cell::RefCell;
use std::future::Future;

thread_local! {
    pub(crate) static LOCAL_POOL: RefCell<LocalPool> = RefCell::new(LocalPool::new());
}

/// Spawns a new local future that will be polled at next tick or a new event comming
pub fn spawn<Fut: Future<Output = ()> + 'static>(future: Fut) -> Result<(), SpawnError> {
    LOCAL_POOL.with(|lp| {
        let lp = lp.borrow();
        lp.spawner().spawn_local(future)
    })
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn on_tick() {
    LOCAL_POOL.with(|lp| {
        let mut lp = lp.borrow_mut();
        lp.run_until_stalled();
    });
}
