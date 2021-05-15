#[cfg(feature = "server")]
pub mod server {
    pub use fivem_server::*;
}

#[cfg(feature = "client")]
pub mod client {
    pub use fivem_client::*;
}

pub mod events {
    pub use fivem_core::events::{emit, subscribe, subscribe_raw, Event, RawEvent};

    #[cfg(feature = "server")]
    pub use fivem_server::emit_net;

    // #[cfg(feature = "client")]
    // pub use fivem_client::emit_net;
}

pub mod runtime {
    pub use fivem_core::runtime::spawn;
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

pub use fivem_core::log;
