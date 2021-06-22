use futures::channel::mpsc::UnboundedSender;
use rustc_hash::FxHashMap;
use std::{borrow::Cow, cell::RefCell, ffi::CStr};

use crate::events::{EventScope, RawEvent, RawEventRef};

pub(crate) struct EventSub {
    pub(crate) scope: EventScope,
    pub(crate) handler: EventHandler,
}

pub(crate) enum EventHandler {
    Future(UnboundedSender<RawEvent>),
    Function(Box<dyn Fn(RawEventRef) + 'static>),
}

// TODO: Vec<UnboundedSender<Event>>
thread_local! {
    pub (crate) static EVENTS: RefCell<FxHashMap<String, EventSub>> = RefCell::new(FxHashMap::default());
}

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

            let event = RawEventRef { source, payload };

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
