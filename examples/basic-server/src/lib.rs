use fivem::{events::EventScope, ref_funcs::RefFunction, server::events::PlayerConnecting};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

async fn handle_connections() {
    let events = fivem::server::events::player_connecting();

    futures::pin_mut!(events);

    while let Some(event) = events.next().await {
        fivem::log(format!(
            "A new player connected: {}. Event source: {:?}",
            event.payload().player_name,
            event.source(),
        ));

        let _ = fivem::runtime::spawn(show_something(event.into_inner()));
    }
}

async fn handle_custom_event() {
    #[derive(Debug, Deserialize)]
    struct Ping {
        req: String,
    }

    #[derive(Serialize)]
    struct Pong((String, u64));

    let mut counter = 0;
    let events = fivem::events::subscribe::<Ping>("client_ping", EventScope::Network);

    futures::pin_mut!(events);

    while let Some(event) = events.next().await {
        let ping = event.payload();

        fivem::log(format!(
            "got a ping from {:?} with message: {:?}",
            event.source(),
            ping.req
        ));

        fivem::events::emit_to_client(
            "server_pong",
            event.source(),
            Pong((ping.req.to_owned(), counter)),
        );

        counter += 1;
    }
}

async fn show_something(event: PlayerConnecting) {
    event.deferrals.defer.invoke::<(), ()>(());

    fivem::runtime::sleep_for(std::time::Duration::from_millis(10)).await;

    #[derive(Serialize)]
    struct UpdateMessage(String);

    #[derive(Serialize)]
    struct DoneMessage(String);

    let udp_msg = UpdateMessage(String::from("Hello from Rust! Wait 5 seconds, please ..."));

    event.deferrals.update.invoke::<(), _>(vec![udp_msg]);
    fivem::runtime::sleep_for(std::time::Duration::from_secs(5)).await;
    event.deferrals.done.invoke::<(), Vec<DoneMessage>>(vec![]);

    // reject a connection
    // let done_msg = DoneMessage(String::from("do not enter!!"));
    // event.deferrals.done.invoke::<(), _>(vec![done_msg]);
}

fn create_export() {
    #[derive(Debug, Deserialize)]
    struct Vector {
        x: f32,
        y: f32,
        z: f32,
    }

    let export = RefFunction::new(|vector: Vec<Vector>| {
        if let Some(vec) = vector.get(0) {
            let length = (vec.x.powi(2) + vec.y.powi(2) + vec.z.powi(2)).sqrt();
            return vec![length];
        }

        vec![0.0]
    });

    fivem::exports::make_export("vecLength", export);
}

async fn test_exports() {
    #[derive(Serialize, Deserialize)]
    struct SomeObject(u32, f32, String);

    // exports("testique", (a, b, c) => console.log(`int: ${a} float: ${b} str: ${c}));
    let testique = fivem::exports::import_function("emitjs", "testique").unwrap();
    testique.invoke::<(), _>(SomeObject(5123, 10.5, String::from("hellow!")));
}

#[no_mangle]
pub extern "C" fn _start() {
    create_export();

    let task = test_exports();
    let _ = fivem::runtime::spawn(task);
    let task = handle_connections();
    let _ = fivem::runtime::spawn(task);
    let task = handle_custom_event();
    let _ = fivem::runtime::spawn(task);
}
