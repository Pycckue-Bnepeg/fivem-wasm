use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    Stream, StreamExt,
};

use serde::{de::DeserializeOwned, Serialize};
use std::{cell::RefCell, collections::HashMap, ffi::CStr};

use crate::invoker::Val;

/// A raw event contains bytes from the emitters.
#[derive(Debug)]
pub struct RawEvent {
    /// A name of an event
    pub name: String,
    /// A source who triggered an event
    pub source: String,
    /// Payload of an event
    pub payload: Vec<u8>,
}

// TODO: Vec<UnboundedSender<Event>>
thread_local! {
    static EVENTS: RefCell<HashMap<String, UnboundedSender<RawEvent>>> = RefCell::new(HashMap::new());
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn __cfx_on_event(
    cstring: *const i8,
    args: *const u8,
    args_length: u32,
    source: *const i8,
) {
    let name = CStr::from_ptr(cstring).to_str().unwrap().to_owned();
    let payload = Vec::from(std::slice::from_raw_parts(args, args_length as _));
    let source = CStr::from_ptr(source).to_str().unwrap().to_owned();

    let event = RawEvent {
        name,
        payload,
        source,
    };

    EVENTS.with(|events| {
        let events = events.borrow();

        if let Some(sender) = events.get(&event.name) {
            let _ = sender.unbounded_send(event);
        }
    });

    crate::runtime::LOCAL_POOL.with(|lp| {
        let mut lp = lp.borrow_mut();
        lp.run_until_stalled();
    });
}

/// A generic event representation.
#[derive(Debug)]
pub struct Event<T: DeserializeOwned> {
    source: String,
    payload: T,
}

impl<T: DeserializeOwned> Event<T> {
    /// Get a source that triggered that event.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Payload of an event
    pub fn payload(&self) -> &T {
        &self.payload
    }

    /// Consumes `Event<T>` and returns inner `T`.
    pub fn into_inner(self) -> T {
        self.payload
    }
}

/// Subscribes on an event with the given name.
///
/// Every time that an event is triggered this function decodes a raw message using messagepack.
///
/// # Example
/// ```rust,ignore
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// struct GiveMoney {
///     to: String, // player name
///     amount: u32 // amount of money
/// }
///
/// let events = fivem::events::subscribe::<GiveMoney>("myCustomEvent");
///
/// while let Some(event) = events.next().await {
///     let source = event.source.clone();
///     let name_from_src = ...;
///     let data = event.into_inner(); // consume an event and take GiveMoney
///     let msg = format!("{} wants to give {} ${}", name_from_src, data.to, data.amount);
/// }
/// # Ok(())
/// # }
///
pub fn subscribe<T: DeserializeOwned>(event_name: &str) -> impl Stream<Item = Event<T>> {
    subscribe_raw(event_name)
        .filter_map(|event| async move {
            rmp_serde::from_read(event.payload.as_slice())
                .ok()
                .map(|payload| (event, payload))
        })
        .map(|(event, payload)| Event {
            source: event.source,
            payload,
        })
}

/// Same as [`subscribe`] but returns [`RawEvent`].
pub fn subscribe_raw(event_name: &str) -> impl Stream<Item = RawEvent> {
    let (tx, rx) = unbounded();

    EVENTS.with(|events| {
        let mut events = events.borrow_mut();
        events.insert(event_name.to_owned(), tx);
    });

    let _ = crate::invoker::register_resource_as_event_handler(event_name);

    rx
}

/// Emits a local event.
pub fn emit<T: Serialize>(event_name: &str, payload: T) {
    if let Ok(payload) = rmp_serde::to_vec_named(&payload) {
        let args = &[
            Val::String(event_name),
            Val::Bytes(&payload),
            Val::Integer(payload.len() as _),
        ];

        let _ = crate::invoker::invoke::<(), _>(0x91310870, args); // TRIGGER_EVENT_INTERNAL
    }
}
