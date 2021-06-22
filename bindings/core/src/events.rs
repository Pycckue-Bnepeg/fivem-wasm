//! Utils to work with CitizenFX events.
//!
//! Currently the best method to use [`subscribe`] (allows you to use it with async/await syntax).
//!
//! Or with [`set_event_handler_closure`]
use futures::{channel::mpsc::unbounded, Future, Stream, StreamExt};

use crate::invoker::Val;
use crate::wasm_impl::events::*;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{borrow::Cow, cell::RefCell, rc::Rc};

/// A raw event contains bytes from the emitters.
#[derive(Debug)]
pub struct RawEvent {
    /// A source who triggered an event
    pub source: String,
    /// Payload of an event
    pub payload: Vec<u8>,
}

pub struct RawEventRef<'a> {
    pub source: Cow<'a, str>,
    pub payload: &'a [u8],
}

impl<'a> RawEventRef<'a> {
    pub(crate) fn to_raw_event(&self) -> RawEvent {
        RawEvent {
            source: self.source.to_string(),
            payload: self.payload.into(),
        }
    }
}

/// An incoming event from CitizenFX.
pub struct Event<'de, T: Deserialize<'de>> {
    source: Cow<'de, str>,
    payload: T,
}

impl<'de, T: Deserialize<'de>> Event<'de, T> {
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

/// Unused for now
pub struct EventOwned<T: DeserializeOwned> {
    source: String,
    payload: T,
}

/// Scope of an event.
///
/// Local events cannot be triggered from network.
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
/// The main difference between [`subscribe`] and [`set_event_handler_closure`] is that
/// the last one calls the passed handler immediately after an event is triggered.
///
/// It is useful for events that contains [`crate::ref_funcs::ExternRefFunction`] to call it.
/// Internaly this function is used in [`crate::exports::make_export`].
pub fn set_event_handler_closure<In, Handler>(event_name: &str, handler: Handler, scope: EventScope)
where
    Handler: Fn(Event<In>) + 'static,
    In: DeserializeOwned,
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

pub trait Handler<Input: DeserializeOwned> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn handle(&mut self, source: String, event: Input) -> Self::Future;
}

pub fn set_event_handler<H, T>(event_name: &str, handler: H, scope: EventScope)
where
    H: Handler<T> + 'static,
    T: DeserializeOwned + 'static,
{
    let handler = Rc::new(RefCell::new(handler));

    let raw_handler = move |raw_event: RawEventRef| {
        let RawEventRef {
            source, payload, ..
        } = raw_event;

        let event = rmp_serde::from_read::<_, T>(payload).ok();

        if let Some(payload) = event {
            // let event = EventOwned {
            //     source: source.to_owned().to_string(),
            //     payload,
            // };

            let handler = handler.clone();
            let source = source.to_string();

            let _ = crate::runtime::spawn(async move {
                let _ = handler.borrow_mut().handle(source, payload);
            });
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

/// Wrapper around a function that implements [`Handler`]
pub struct HandlerFn<T> {
    func: T,
}

/// Wraps an `async` function into [`HandlerFn`] object.
///
/// # Example
/// ```rust,ignore
/// use cfx::events::*;
///
/// #[derive(Debug, Deserialize)]
/// struct SomeEvent(String)
///
/// async fn handle_event(source: String, event: SomeEvent) -> Result<(), ()> {
///     cfx::log(format!("got an event with: {}", event.0));
///     Ok(())
/// }
///
/// // wrap our function into Handler
/// let handler = handler_fn(handle_event);
///
/// set_event_handler(
///     "someEvent",
///     handler,
///     EventScope::Local,
/// );
/// ```
pub fn handler_fn<T>(func: T) -> HandlerFn<T> {
    HandlerFn { func }
}

impl<T, F, Input, R, E> Handler<Input> for HandlerFn<T>
where
    T: FnMut(String, Input) -> F,
    F: Future<Output = Result<R, E>>,
    Input: DeserializeOwned,
{
    type Response = R;
    type Error = E;
    type Future = F;

    fn handle(&mut self, source: String, event: Input) -> Self::Future {
        (self.func)(source, event)
    }
}
