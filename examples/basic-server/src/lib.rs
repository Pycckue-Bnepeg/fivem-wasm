use fivem::{ref_funcs::RefFunction, server::events::PlayerConnecting};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

async fn handle_connections() {
    let mut events = fivem::server::events::player_connecting();

    while let Some(event) = events.next().await {
        fivem::log(format!(
            "A new player connected: {}",
            event.payload().player_name
        ));

        let _ = fivem::runtime::spawn(show_something(event.into_inner()));
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

mod kvp {
    use fivem::invoker::{invoke, Val};

    pub fn delete_resource_kvp(key: &str) {
        let _ = invoke::<(), _>(0x7389B5DF, &[Val::String(key)]);
    }

    pub fn end_find_kvp(handle: u32) {
        let _ = invoke::<(), _>(0xB3210203, &[Val::Integer(handle)]);
    }
    pub fn find_kvp(handle: u32) -> Option<String> {
        invoke(0xBD7BEBC5, &[Val::Integer(handle)]).ok()
    }

    pub fn resource_kvp_float(key: &str) -> Option<f32> {
        invoke(0x35BDCEEA, &[Val::String(key)]).ok()
    }

    pub fn resource_kvp_int(key: &str) -> Option<u32> {
        invoke(0x557B586A, &[Val::String(key)]).ok()
    }

    pub fn resource_kvp_string(key: &str) -> Option<String> {
        invoke(0x5240DA5A, &[Val::String(key)]).ok()
    }

    pub fn set_resource_kvp_float(key: &str, val: f32) {
        let _ = invoke::<(), _>(0x9ADD2938, &[Val::String(key), Val::Float(val)]);
    }

    pub fn set_resource_kvp_int(key: &str, val: u32) {
        let _ = invoke::<(), _>(0x6A2B1E8, &[Val::String(key), Val::Integer(val)]);
    }

    pub fn set_resource_kvp(key: &str, val: &str) {
        let _ = invoke::<(), _>(0x21C7A35B, &[Val::String(key), Val::String(val)]);
    }

    pub fn start_find_kvp(prefix: &str) -> Option<u32> {
        invoke(0xDD379006, &[Val::String(prefix)]).ok()
    }
}

fn print_my_keys() {
    println!("START FINDING KEYS:");

    if let Some(handle) = kvp::start_find_kvp("my:") {
        while let Some(key) = kvp::find_kvp(handle) {
            println!("found a new key: {:?}", key);
        }

        kvp::end_find_kvp(handle);
    }

    println!("DONE FINDING KEYS");
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
    // cleanup prev
    kvp::delete_resource_kvp("my:int");
    kvp::delete_resource_kvp("my:str");
    kvp::delete_resource_kvp("my:float");

    println!("BEFORE:");

    println!("{:?}", kvp::resource_kvp_int("my:int"));
    println!("{:?}", kvp::resource_kvp_string("my:str"));
    println!("{:?}", kvp::resource_kvp_float("my:float"));

    kvp::set_resource_kvp("my:str", "stringify");
    kvp::set_resource_kvp_float("my:float", 1345.5);
    kvp::set_resource_kvp_int("my:int", 55561);

    println!("AFTER:");

    println!("{:?}", kvp::resource_kvp_int("my:int"));
    println!("{:?}", kvp::resource_kvp_string("my:str"));
    println!("{:?}", kvp::resource_kvp_float("my:float"));

    print_my_keys();
    create_export();

    let task = test_exports();
    let _ = fivem::runtime::spawn(task);
    let task = handle_connections();
    let _ = fivem::runtime::spawn(task);
}
