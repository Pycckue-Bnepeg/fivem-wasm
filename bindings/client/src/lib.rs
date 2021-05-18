use fivem_core::invoker::{invoke, Val};
use serde::Serialize;

pub mod natives;

pub mod events {
    use fivem_core::events::Event;

    use futures::{Stream, StreamExt};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ClientGameTypeStart {
        pub resource_name: String,
    }

    pub fn client_game_type_start() -> impl Stream<Item = Event<ClientGameTypeStart>> {
        fivem_core::events::subscribe("onClientGameTypeStart").boxed_local()
    }
}

pub fn emit_net<T: Serialize>(event_name: &str, payload: T) {
    if let Ok(payload) = rmp_serde::to_vec(&payload) {
        let args = &[
            Val::String(event_name),
            Val::Bytes(&payload),
            Val::Integer(payload.len() as _),
        ];

        let _ = invoke::<(), _>(0x7FDD1128, args); // TRIGGER_SERVER_EVENT_INTERNAL
    }
}
