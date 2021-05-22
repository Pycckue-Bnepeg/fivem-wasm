#[cfg(feature = "server")]
pub mod server {
    pub use fivem_server::natives::*;
    pub use fivem_server::{emit_net, events};
}

#[cfg(feature = "client")]
pub mod client {
    pub use fivem_client::natives::*;
    pub use fivem_client::{emit_net, events, TaskSequenceBuilder};
}

pub mod events {
    pub use fivem_core::events::{
        emit, set_event_handler, set_event_handler_test, subscribe, subscribe_raw, Event,
        EventScope, Handler, RawEvent,
    };

    #[cfg(feature = "server")]
    pub use fivem_server::emit_net as emit_to_client;

    #[cfg(feature = "client")]
    pub use fivem_client::emit_net as emit_to_server;
}

pub mod runtime {
    pub use fivem_core::runtime::{sleep_for, spawn};
}

pub mod invoker {
    pub use fivem_core::invoker::{invoke, InvokeError, Val};
}

pub mod types {
    pub use fivem_core::types::{Packed, Vector3};
}

pub mod ref_funcs {
    pub use fivem_core::ref_funcs::{ExternRefFunction, RefFunction};
}

pub mod exports {
    pub use fivem_core::exports::{import_function, make_export};
}

pub use fivem_core::log;
