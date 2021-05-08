use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    Stream, StreamExt,
};

use serde::{de::DeserializeOwned, Serialize};
use std::{cell::RefCell, collections::HashMap, ffi::CStr};

use crate::invoker::Val;

#[derive(Debug)]
struct InternalEvent {
    name: String,
    source: String,
    payload: Vec<u8>,
}

// TODO: Vec<UnboundedSender<Event>>
thread_local! {
    static EVENTS: RefCell<HashMap<String, UnboundedSender<InternalEvent>>> = RefCell::new(HashMap::new());
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn on_event(
    cstring: *const i8,
    args: *const u8,
    args_length: u32,
    source: *const i8,
) {
    let name = CStr::from_ptr(cstring).to_str().unwrap().to_owned();
    let payload = Vec::from(std::slice::from_raw_parts(args, args_length as _));
    let source = CStr::from_ptr(source).to_str().unwrap().to_owned();

    let event = InternalEvent {
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

    pub fn payload(&self) -> &T {
        &self.payload
    }

    pub fn into_inner(self) -> T {
        self.payload
    }
}

pub fn subscribe<T: DeserializeOwned>(event_name: &str) -> impl Stream<Item = Event<T>> {
    let (tx, rx) = unbounded();

    EVENTS.with(|events| {
        let mut events = events.borrow_mut();
        events.insert(event_name.to_owned(), tx);
    });

    crate::invoker::register_resource_as_event_handler(event_name);

    rx.filter_map(|event| async move {
        rmp_serde::from_read(event.payload.as_slice())
            .ok()
            .map(|payload| (event, payload))
    })
    .map(|(event, payload)| Event {
        source: event.source,
        payload,
    })
}

pub fn emit<T: Serialize>(event_name: &str, payload: T) {
    if let Ok(payload) = rmp_serde::to_vec(&payload) {
        let args = &[
            Val::String(event_name),
            Val::Bytes(&payload),
            Val::Integer(payload.len() as _),
        ];

        crate::invoker::invoke::<(), _>(0x91310870, args); // TRIGGER_EVENT_INTERNAL
    }
}

pub fn emit_net<T: Serialize>(event_name: &str, source: &str, payload: T) {
    if let Ok(payload) = rmp_serde::to_vec(&payload) {
        let args = &[
            Val::String(event_name),
            Val::String(source),
            Val::Bytes(&payload),
            Val::Integer(payload.len() as _),
        ];

        crate::invoker::invoke::<(), _>(0x2F7A49E6, args); // TRIGGER_CLIENT_EVENT_INTERNAL
    }
}
