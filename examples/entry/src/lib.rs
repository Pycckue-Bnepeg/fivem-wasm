use fivem_bindings::log;
use futures::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct ServerResourceStart {
    resource_name: String,
}

#[no_mangle]
pub extern "C" fn _start() {
    let mut events =
        fivem_bindings::events::subscribe::<ServerResourceStart>("onServerResourceStart")
            .boxed_local();

    let task = async move {
        while let Some(event) = events.next().await {
            log(format!(
                "new resource started: {}",
                event.payload().resource_name
            ));
        }
    };

    let res = fivem_bindings::runtime::spawn(task);

    log(format!("cool suck me. {:?}", res));
}
