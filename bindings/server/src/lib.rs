use serde::Serialize;

pub mod natives;

pub mod events {
    use fivem_core::events::Event;
    use fivem_core::ref_funcs::ExternRefFunction;
    use futures::Stream;
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

    pub fn player_connecting<'a>() -> impl Stream<Item = Event<'a, PlayerConnecting>> {
        fivem_core::events::subscribe("playerConnecting", fivem_core::events::EventScope::Local)
    }
}

pub fn emit_net<T: Serialize>(event_name: &str, source: &str, payload: T) {
    if let Ok(payload) = rmp_serde::to_vec(&payload) {
        natives::cfx::trigger_client_event_internal(
            event_name,
            source,
            payload.as_slice(),
            payload.len() as _,
        );
    }
}
