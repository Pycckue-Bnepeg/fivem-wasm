#[cfg(feature = "server")]
pub mod server {
    pub use cfx_server::natives::*;
    pub use cfx_server::{emit_net, events};
}

#[cfg(feature = "client")]
pub mod client {
    pub use cfx_client::natives::*;
    pub use cfx_client::{emit_net, events, TaskSequenceBuilder};
}

pub mod events {
    pub use cfx_core::events::{
        emit, handler_fn, set_event_handler, set_event_handler_closure, subscribe, subscribe_raw,
        Event, EventScope, Handler, HandlerFn, RawEvent,
    };

    #[cfg(feature = "server")]
    pub use cfx_server::emit_net as emit_to_client;

    #[cfg(feature = "client")]
    pub use cfx_client::emit_net as emit_to_server;
}

pub mod runtime {
    pub use cfx_core::runtime::{sleep_for, spawn};
}

pub mod invoker {
    pub use cfx_core::invoker::{invoke, InvokeError, Val};
}

pub mod types {
    pub use cfx_core::types::{Packed, Vector3};
}

pub mod ref_funcs {
    pub use cfx_core::ref_funcs::{ExternRefFunction, RefFunction};
}

pub mod exports {
    pub use cfx_core::exports::{import_function, make_export};
}

pub use cfx_core::log;
