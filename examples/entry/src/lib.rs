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

fn registered_commands() -> Vec<Commands> {
    // 0xD4BEF069
    let list: Packed<Vec<Commands>> = fivem::invoker::invoke(0xD4BEF069, &[]).unwrap();
    list.into_inner()
}

fn register_console_listener() {
    #[derive(Deserialize)]
    struct Test {
        channel: String,
        message: String,
    }

    let func = fivem::ref_funcs::RefFunction::new(|i: Test| -> () {
        println!("WOW !");
        ()
    });

    let _ =
        fivem::invoker::invoke::<(), _>(0x281B5448, &[fivem::invoker::Val::RefFunc(func.clone())]);
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

    // let commands = registered_commands();
    register_console_listener();

    // log(format!("{:?}", commands));
}
