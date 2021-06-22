use cfx_core::events::Event;
use futures::Stream;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientGameTypeStart {
    pub resource_name: String,
}

pub fn client_game_type_start<'a>() -> impl Stream<Item = Event<'a, ClientGameTypeStart>> {
    cfx_core::events::subscribe("onClientGameTypeStart", cfx_core::events::EventScope::Local)
}
