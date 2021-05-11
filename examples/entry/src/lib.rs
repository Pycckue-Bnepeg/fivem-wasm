use fivem::{log, types::Packed};
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

#[derive(Debug, Deserialize)]
struct Commands {
    name: String,
}

// TODO: Ref funcs
// fn register_command() {
//     // 0x5FA79B0F
// }

fn registered_commands() -> Vec<Commands> {
    // 0xD4BEF069
    let list: Packed<Vec<Commands>> = fivem::invoker::invoke(0xD4BEF069, &[]).unwrap();
    list.into_inner()
}

#[no_mangle]
pub extern "C" fn _start() {
    let start_events = fivem::events::subscribe::<ServerResourceStart>("onServerResourceStart")
        .map(|ev| Resource::Start(ev.into_inner().resource_name))
        .boxed_local();

    let stop_events = fivem::events::subscribe::<ServerResourceStop>("onServerResourceStop")
        .map(|ev| Resource::Stop(ev.into_inner().resource_name))
        .boxed_local();

    let mut resources = futures::stream::select(start_events, stop_events);

    let task = async move {
        while let Some(event) = resources.next().await {
            log(event.readable());
        }
    };

    let _ = fivem::runtime::spawn(task);

    let commands = registered_commands();

    log(format!("{:?}", commands));
}
