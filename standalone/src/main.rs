use cfx_wasm_runtime::Runtime;

fn main() {
    let mut rt = Runtime::new();
    let bytes = load_module("./target/wasm32-wasi/release/entry.wasi.wasm");

    rt.load_module(&bytes, true);
    // rt.trigger_event("some_cool_event", 228);
}

fn load_module(path: &str) -> Vec<u8> {
    use std::io::*;

    let mut file = std::fs::File::open(path).unwrap();
    let mut buf = Vec::new();

    file.read_to_end(&mut buf).unwrap();

    return buf;
}
