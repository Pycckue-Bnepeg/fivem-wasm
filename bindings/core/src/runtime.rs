use futures::{
    channel::oneshot,
    task::{LocalSpawnExt, SpawnError},
    Future, TryFutureExt,
};

use std::time::{Duration, Instant};

pub(crate) use crate::wasm_impl::runtime::LOCAL_POOL;
use crate::wasm_impl::runtime::{SPAWNER, TIMERS};

/// Spawns a new local future that will be polled at next tick or a new event comming
pub fn spawn<Fut: Future<Output = ()> + 'static>(future: Fut) -> Result<(), SpawnError> {
    SPAWNER.with(|sp| sp.borrow().spawn_local(future))
}

/// Stops execution for duration (doesn't block CitizenFX).
///
/// # Example
/// ```rust,ignore
/// // called when a player is connecting to our server
/// async fn show_something(event: PlayerConnecting) {
///     event.deferrals.defer.invoke::<(), ()>(());
///
///     cfx::runtime::sleep_for(std::time::Duration::from_millis(10)).await; // CitizenFX wants to wait at least one server tick.
///
///     #[derive(Serialize)]
///     struct UpdateMessage(String);
///
///     #[derive(Serialize)]
///     struct DoneMessage(String);
///
///     let udp_msg = UpdateMessage(String::from("Hello from Rust! Wait 5 seconds, please ..."));
///
///     // send a welcome message to a player
///     event.deferrals.update.invoke::<(), _>(vec![udp_msg]);
///
///     // and now wait 5 sec
///     cfx::runtime::sleep_for(std::time::Duration::from_secs(5)).await;
///
///     // allow user to connect
///     event.deferrals.done.invoke::<(), Vec<DoneMessage>>(vec![]);
///
///     // reject a connection
///     // let done_msg = DoneMessage(String::from("do not enter!!"));
///     // event.deferrals.done.invoke::<(), _>(vec![done_msg]);
/// }
/// ```
pub fn sleep_for(duration: Duration) -> impl Future<Output = ()> {
    let instant = Instant::now().checked_add(duration).unwrap();
    let (tx, rx) = oneshot::channel();

    TIMERS.with(|timers| {
        let mut timers = timers.borrow_mut();
        let entry = timers.entry(instant).or_insert_with(Vec::new);
        entry.push(tx);
    });

    rx.unwrap_or_else(|_| ())
}
