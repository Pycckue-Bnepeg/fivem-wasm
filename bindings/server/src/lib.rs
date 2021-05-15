use fivem_core::invoker::{invoke, Val};
use serde::Serialize;

pub mod events {
    use fivem_core::events::Event;
    use fivem_core::ref_funcs::ExternRefFunction;

    use futures::{Stream, StreamExt};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Deferrals {
        pub defer: ExternRefFunction,
        pub done: ExternRefFunction,
        pub handover: ExternRefFunction,
        #[serde(rename = "presentCard")]
        pub present_card: ExternRefFunction,
        pub update: ExternRefFunction,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PlayerConnecting {
        pub player_name: String,
        pub set_kick_reason: ExternRefFunction,
        pub deferrals: Deferrals,
        // source: String,
    }

    pub fn player_connecting() -> impl Stream<Item = Event<PlayerConnecting>> {
        fivem_core::events::subscribe("playerConnecting").boxed_local()
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

        let _ = invoke::<(), _>(0x2F7A49E6, args); // TRIGGER_CLIENT_EVENT_INTERNAL
    }
}
