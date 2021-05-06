use fivem_bindings::log;
use futures::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct ServerResourceStart {
    resource_name: String,
}

#[derive(Deserialize)]
struct ServerResourceStop {
    resource_name: String,
}

enum Resource {
    Start(String),
    Stop(String),
}

impl Resource {
    fn readable(self) -> String {
        match self {
            Resource::Start(start) => format!("resource started: {:?}", start),
            Resource::Stop(stop) => format!("resource stopped: {:?}", stop),
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() {
    let start_events =
        fivem_bindings::events::subscribe::<ServerResourceStart>("onServerResourceStart")
            .map(|ev| Resource::Start(ev.payload().resource_name.clone()))
            .boxed_local();

    let stop_events =
        fivem_bindings::events::subscribe::<ServerResourceStart>("onServerResourceStop")
            .map(|ev| Resource::Stop(ev.payload().resource_name.clone()))
            .boxed_local();

    let mut resources = futures::stream::select(start_events, stop_events);

    let task = async move {
        while let Some(event) = resources.next().await {
            log(event.readable());
        }
    };

    let res = fivem_bindings::runtime::spawn(task);

    log(format!("cool suck me. {:?}", res));
}
