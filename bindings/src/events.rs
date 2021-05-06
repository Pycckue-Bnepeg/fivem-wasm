use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    Stream, StreamExt,
};

use serde::de::DeserializeOwned;

use std::{cell::RefCell, collections::HashMap, ffi::CStr};

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

#[derive(Debug)]
pub struct Event<T: DeserializeOwned> {
    source: String,
    payload: T,
}

impl<T: DeserializeOwned> Event<T> {
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn payload(&self) -> &T {
        &self.payload
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
