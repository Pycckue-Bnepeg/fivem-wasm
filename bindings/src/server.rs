pub mod events {
    use crate::events::Event;
    use crate::ref_funcs::ExternRefFunction;
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
        crate::events::subscribe("playerConnecting").boxed_local()
    }
}
