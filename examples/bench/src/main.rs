use fivem::ref_funcs::RefFunction;

macro_rules! log {
    () => (fivem::log("\n"));
    ($($arg:tt)*) => ({
        fivem::log(std::format_args!($($arg)*).to_string());
    })
}

fn bench_exports() {
    use easybench::bench;
    use fivem::exports::import_function;

    // exports are listening to an event with a special name
    // and calling the passed function
    let wasm_export = import_function("wasmbench", "exportBench").unwrap();
    let js_export = import_function("jsbench", "exportBench").unwrap();

    log!("{}", bench(|| wasm_export.invoke::<(), _>(vec![0u32])));
    log!("{}", bench(|| js_export.invoke::<(), _>(vec![0u32])));
}

fn main() {
    // startup
    fn create_export() {
        let func = RefFunction::new(|_: Vec<u32>| {});
        fivem::exports::make_export("exportBench", func);
    }

    create_export();

    // benchmarks
    bench_exports();
}
