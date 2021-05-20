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

struct EventSub {
    scope: EventScope,
    handler: EventHandler,
}

enum EventHandler {
    Future(UnboundedSender<RawEvent>),
    Function(Box<dyn FnMut(RawEvent) + 'static>),
}

// TODO: Vec<UnboundedSender<Event>>
thread_local! {
    // static EVENTS: RefCell<HashMap<String, UnboundedSender<RawEvent>>> = RefCell::new(HashMap::new());
    static EVENTS: RefCell<HashMap<String, EventSub>> = RefCell::new(HashMap::new());
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

    let mut event = RawEvent {
        name,
        payload,
        source,
    };

    EVENTS.with(|events| {
        let mut events = events.borrow_mut();

        if let Some(sub) = events.get_mut(&event.name) {
            if event.source.starts_with("net:") {
                if sub.scope != EventScope::Network {
                    return;
                }

                event.source = event.source.strip_prefix("net:").unwrap().to_owned();
            } else if
            /* is_duplicity_version && */
            event.source.starts_with("internal-net:") {
                event.source = event
                    .source
                    .strip_prefix("internal-net:")
                    .unwrap()
                    .to_owned();
            }

            match sub.handler {
                EventHandler::Function(ref mut func) => {
                    func(event);
                }
                EventHandler::Future(ref sender) => {
                    let _ = sender.unbounded_send(event);
                }
            }
        }
    });

    crate::runtime::LOCAL_POOL.with(|lp| {
        if let Ok(mut lp) = lp.try_borrow_mut() {
            lp.run_until_stalled();
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventScope {
    Local,
    Network,
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
pub fn subscribe<T: DeserializeOwned>(
    event_name: &str,
    scope: EventScope,
) -> impl Stream<Item = Event<T>> {
    subscribe_raw(event_name, scope)
        .filter_map(|event| async move {
            rmp_serde::from_read(event.payload.as_slice())
                .ok()
                .map(|payload| (event, payload))
        })
        .map(|(event, payload)| Event {
            source: event.source,
            payload,
        })
        .boxed_local()
}

/// Same as [`subscribe`] but returns [`RawEvent`].
pub fn subscribe_raw(event_name: &str, scope: EventScope) -> impl Stream<Item = RawEvent> {
    let (tx, rx) = unbounded();

    EVENTS.with(|events| {
        let sub = EventSub {
            scope,
            handler: EventHandler::Future(tx),
        };

        let mut events = events.borrow_mut();
        events.insert(event_name.to_owned(), sub);
    });

    let _ = crate::invoker::register_resource_as_event_handler(event_name);

    rx.boxed_local()
}

/// Sets an event handler.
///
/// The main difference between [`subscribe`] and [`set_event_handler`] is that
/// the last one calls the passed handler immediately after an event is triggered.
///
/// It is useful for events that contains [`crate::ref_funcs::ExternRefFunction`] to call it.
/// Internaly this function is used in [`crate::exports::make_export`].
pub fn set_event_handler<In, Handler>(event_name: &str, mut handler: Handler, scope: EventScope)
where
    Handler: FnMut(Event<In>) + 'static,
    In: DeserializeOwned,
{
    let raw_handler = move |raw_event: RawEvent| {
        let event = rmp_serde::from_read(raw_event.payload.as_slice())
            .ok()
            .map(|payload| (raw_event, payload));

        if let Some((event, payload)) = event {
            let event = Event {
                source: event.source,
                payload,
            };

            handler(event);
        }
    };

    EVENTS.with(|events| {
        let sub = EventSub {
            scope,
            handler: EventHandler::Function(Box::new(raw_handler)),
        };

        let mut events = events.borrow_mut();
        events.insert(event_name.to_owned(), sub);
    });

    let _ = crate::invoker::register_resource_as_event_handler(event_name);
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
