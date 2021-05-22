use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    Stream, StreamExt,
};

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::RefCell, ffi::CStr};

use rustc_hash::FxHashMap;

use crate::invoker::Val;

/// A raw event contains bytes from the emitters.
#[derive(Debug)]
pub struct RawEvent {
    /// A source who triggered an event
    pub source: String,
    /// Payload of an event
    pub payload: Vec<u8>,
}

pub struct RawEventRef<'a> {
    source: Cow<'a, str>,
    payload: &'a [u8],
}

impl<'a> RawEventRef<'a> {
    fn to_raw_event(&self) -> RawEvent {
        RawEvent {
            source: self.source.to_string(),
            payload: self.payload.into(),
        }
    }
}

struct EventSub {
    scope: EventScope,
    handler: EventHandler,
}

enum EventHandler {
    Future(UnboundedSender<RawEvent>),
    Function(Box<dyn Fn(RawEventRef) + 'static>),
}

// TODO: Vec<UnboundedSender<Event>>
thread_local! {
    // static EVENTS: RefCell<HashMap<String, UnboundedSender<RawEvent>>> = RefCell::new(HashMap::new());
    static EVENTS: RefCell<FxHashMap<String, EventSub>> = RefCell::new(FxHashMap::default());
}

#[doc(hidden)]
#[no_mangle]
pub unsafe extern "C" fn __cfx_on_event(
    cstring: *const i8,
    args: *const u8,
    args_length: u32,
    source: *const i8,
) {
    let name = CStr::from_ptr(cstring).to_str().unwrap();
    let payload = std::slice::from_raw_parts(args, args_length as _);
    let source = CStr::from_ptr(source).to_str().unwrap();

    EVENTS.with(|events| {
        let events = events.borrow();

        if let Some(sub) = events.get(name) {
            let source = if source.starts_with("net:") {
                if sub.scope != EventScope::Network {
                    return;
                }

                Cow::from(source.strip_prefix("net:").unwrap())
            } else if
            /* is_duplicity_version && */
            source.starts_with("internal-net:") {
                Cow::from(source.strip_prefix("internal-net:").unwrap())
            } else {
                Cow::from("")
            };

            let event = RawEventRef {
                source: Cow::from(source),
                payload,
            };

            match sub.handler {
                EventHandler::Function(ref func) => {
                    func(event);
                }

                EventHandler::Future(ref sender) => {
                    let _ = sender.unbounded_send(event.to_raw_event());
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

/// Same as [`Event`] but without clone and allocations (for [`set_event_handler`]).
pub struct Event<'de, T: Deserialize<'de> + 'de> {
    source: Cow<'de, str>,
    payload: T,
}

impl<'de, T: Deserialize<'de> + 'de> Event<'de, T> {
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
pub fn subscribe<'a, In>(event_name: &'a str, scope: EventScope) -> impl Stream<Item = Event<In>>
where
    for<'de> In: Deserialize<'de> + 'a,
{
    let mut events = subscribe_raw(event_name, scope);

    async_stream::stream! {
        while let Some(event) = events.next().await {
            if let Ok(payload) = rmp_serde::from_read_ref(&event.payload) {
                let event = Event {
                    source: Cow::from(event.source),
                    payload,
                };

                yield event;
            }
        }
    }
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

    rx
}

/// Sets an event handler.
///
/// The main difference between [`subscribe`] and [`set_event_handler`] is that
/// the last one calls the passed handler immediately after an event is triggered.
///
/// It is useful for events that contains [`crate::ref_funcs::ExternRefFunction`] to call it.
/// Internaly this function is used in [`crate::exports::make_export`].
pub fn set_event_handler<In, Handler>(event_name: &str, handler: Handler, scope: EventScope)
where
    Handler: Fn(Event<In>) + 'static,
    for<'de> In: Deserialize<'de> + 'de,
{
    let raw_handler = move |raw_event: RawEventRef| {
        let RawEventRef {
            source, payload, ..
        } = raw_event;

        let event = rmp_serde::from_read_ref::<_, In>(&payload).ok();

        if let Some(payload) = event {
            let event = Event { source, payload };

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
