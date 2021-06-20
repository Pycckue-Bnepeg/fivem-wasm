use cfx::{events::Event, ref_funcs::RefFunction};
use easybench::bench;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

const LONG_STRING: &str = include_str!("long.str");
const SHORT_STRING: &str = "hello!";

macro_rules! log {
    () => (cfx::log("\n"));
    ($($arg:tt)*) => ({
        cfx::log(std::format_args!($($arg)*).to_string());
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct CustomEvent {
    int: u32,
    string: String,
}

fn bench_invoking() {
    use cfx::server::cfx::*;

    log!(
        "bench_invoking::wasm::get_num_resources {}",
        bench(|| get_num_resources())
    );
    log!(
        "bench_invoking::wasm::cancel_event {}",
        bench(|| cancel_event())
    );
}

fn bench_exports() {
    use cfx::exports::import_function;

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
    use cfx::events::emit;

    #[derive(Debug, Serialize)]
    struct CustomEvent((u32, &'static str));

    log!(
        "bench_event_handler::wasm_sync_closure (long) {}",
        bench(|| emit("wasmEventHandlerClosure", CustomEvent((512, LONG_STRING))))
    );

    log!(
        "bench_event_handler::wasm_sync (long) {}",
        bench(|| emit("wasmEventHandler", CustomEvent((512, LONG_STRING))))
    );

    log!(
        "bench_event_handler::wasm_async (long) {}",
        bench(|| emit("wasmEventHandlerAsync", CustomEvent((512, LONG_STRING))))
    );

    log!(
        "bench_event_handler::js (long) {}",
        bench(|| emit("jsEventHandler", CustomEvent((512, LONG_STRING))))
    );

    log!(
        "bench_event_handler::wasm_sync_closure (short) {}",
        bench(|| emit("wasmEventHandlerClosure", CustomEvent((256, SHORT_STRING))))
    );

    log!(
        "bench_event_handler::wasm_sync (short) {}",
        bench(|| emit("wasmEventHandler", CustomEvent((256, SHORT_STRING))))
    );

    log!(
        "bench_event_handler::wasm_async (short) {}",
        bench(|| emit("wasmEventHandlerAsync", CustomEvent((256, SHORT_STRING))))
    );

    log!(
        "bench_event_handler::js (short) {}",
        bench(|| emit("jsEventHandler", CustomEvent((256, SHORT_STRING))))
    );
}

async fn event_handle(source: String, event: CustomEvent) -> Result<(), ()> {
    Ok(())
}

fn main() {
    // startup
    fn create_export() {
        let func = RefFunction::new(|_: Vec<u32>| {});
        cfx::exports::make_export("exportBench", func);
    }

    fn set_event_handler() {
        cfx::events::set_event_handler(
            "wasmEventHandler",
            cfx::events::handler_fn(event_handle),
            cfx::events::EventScope::Local,
        );

        cfx::events::set_event_handler_closure(
            "wasmEventHandlerClosure",
            |_ev: Event<CustomEvent>| {},
            cfx::events::EventScope::Local,
        );

        let events = cfx::events::subscribe::<CustomEvent>(
            "wasmEventHandlerAsync",
            cfx::events::EventScope::Local,
        );

        let task = async move {
            futures::pin_mut!(events);

            while let Some(_ev) = events.next().await {}
        };

        let _ = cfx::runtime::spawn(task);
    }

    create_export();
    set_event_handler();

    // start jsbench; start wasmbench;
    // refresh; restart jsbench; restart wasmbench;

    // benchmarks
    bench_exports();
    bench_event_handler();
    bench_invoking();
}

/*

i7-8700k + nvme ssd

[    script:wasmbench] bench_exports::wasm                                  1.692µs (R²=0.997, 633203 iterations in 115 samples)
[    script:wasmbench] bench_exports::js                                    6.296µs (R²=0.992, 151574 iterations in 100 samples)
[    script:wasmbench] bench_event_handler::wasm_sync_closure (long)        7.553µs (R²=0.993, 137793 iterations in 99 samples)
[    script:wasmbench] bench_event_handler::wasm_sync (long)                5.541µs (R²=0.997, 201750 iterations in 103 samples)
[    script:wasmbench] bench_event_handler::wasm_async (long)               8.87µs (R²=0.995, 113876 iterations in 97 samples)
[    script:wasmbench] bench_event_handler::js (long)                       60.708µs (R²=0.992, 15376 iterations in 76 samples)
[    script:wasmbench] bench_event_handler::wasm_sync_closure (short)       2.985µs (R²=0.998, 357422 iterations in 109 samples)
[    script:wasmbench] bench_event_handler::wasm_sync (short)               2.687µs (R²=0.996, 393165 iterations in 110 samples)
[    script:wasmbench] bench_event_handler::wasm_async (short)              3.221µs (R²=0.997, 324928 iterations in 108 samples)
[    script:wasmbench] bench_event_handler::js (short)                      7.778µs (R²=0.997, 137793 iterations in 99 samples)
[    script:wasmbench] bench_invoking::wasm::get_num_resources              655ns (R²=0.999, 1642386 iterations in 125 samples)
[    script:wasmbench] bench_invoking::wasm::cancel_event                   202ns (R²=0.997, 5154537 iterations in 137 samples)

*/
