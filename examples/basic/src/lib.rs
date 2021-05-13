use futures::StreamExt;

async fn handle_connections() {
    let mut events = fivem::server::events::player_connecting();

    while let Some(event) = events.next().await {
        fivem::log(format!(
            "A new player connected: {}",
            event.payload().player_name
        ));
    }
}

#[no_mangle]
pub extern "C" fn _start() {
    let task = handle_connections();
    let _ = fivem::runtime::spawn(task);
}
