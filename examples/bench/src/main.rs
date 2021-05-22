use easybench::bench;
use fivem::{events::Event, ref_funcs::RefFunction};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

macro_rules! log {
    () => (fivem::log("\n"));
    ($($arg:tt)*) => ({
        fivem::log(std::format_args!($($arg)*).to_string());
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct CustomEvent {
    int: u32,
    string: String,
}

fn bench_exports() {
    use fivem::exports::import_function;

    // exports are listening to an event with a special name
    // and calling the passed function
    let wasm_export = import_function("wasmbench", "exportBench").unwrap();
    let js_export = import_function("jsbench", "exportBench").unwrap();

    log!(
        "bench_exports::wasm {}",
        bench(|| wasm_export.invoke::<(), _>(vec![0u32]))
    );

    log!(
        "bench_exports::js {}",
        bench(|| js_export.invoke::<(), _>(vec![0u32]))
    );
}

fn bench_event_handler() {
    use fivem::events::emit;

    #[derive(Debug, Serialize)]
    struct CustomEvent((u32, &'static str));

    log!(
        "bench_event_handler::wasm_sync {}",
        bench(|| emit("wasmEventHandler", CustomEvent((512, "hi!"))))
    );

    log!(
        "bench_event_handler::wasm_async {}",
        bench(|| emit("wasmEventHandlerAsync", CustomEvent((512, "hi!"))))
    );

    log!(
        "bench_event_handler::js {}",
        bench(|| emit("jsEventHandler", CustomEvent((512, "hi!"))))
    );
}

fn main() {
    // startup
    fn create_export() {
        let func = RefFunction::new(|_: Vec<u32>| {});
        fivem::exports::make_export("exportBench", func);
    }

    fn set_event_handler() {
        fivem::events::set_event_handler(
            "wasmEventHandler",
            |_ev: Event<CustomEvent>| {},
            fivem::events::EventScope::Local,
        );

        let events = fivem::events::subscribe::<CustomEvent>(
            "wasmEventHandlerAsync",
            fivem::events::EventScope::Local,
        );

        let task = async move {
            futures::pin_mut!(events);

            while let Some(_ev) = events.next().await {}
        };

        let _ = fivem::runtime::spawn(task);
    }

    create_export();
    set_event_handler();

    // start jsbench; start wasmbench;
    // refresh; restart jsbench; restart wasmbench;

    // benchmarks
    bench_exports();
    bench_event_handler();
}
