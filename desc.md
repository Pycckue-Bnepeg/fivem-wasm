# Summary

The goal of this project is make WebAssembly runtime for FiveM. With this runtime we add ability for end-users to make fast scripts with languages supporting compiling in WASM (like Rust, Go, C, AssemblyScript). It is an opportunity to bring in new developers and programming paradigms.

# Explanation

Current functionality:

- Invoke native functions
- Subscribe to events
- Call ref functions
- Communicate with another runtimes / scripts.

There is already the v8 engine that is capable of using WebAssembly modules. When we develop a script we mostly use native functions, so, if we choose to write bindings for JS it makes no sense, we will spend all the time in JS runtime and convert objects from JS to our RT and back. It is costly and ugly method. Also most of the mentioned languages is strongly typed and this requires to make some hack for functions like `console.log` (just look at [`web-sys`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/console/index.html) Rust crate), `emit`, callbacks in `on/onNet`. If I got it right currently FiveM contains old version of Node.js (~12) and v8 itself so it is better to have one shared runtime that can be updated by change one line Cargo.toml.

With this implementation of WASM scripting runtime we remove the JS (v8) layer and deal directly with FiveM. We don't forced to make some wrappers to work with JS functions, no variadic functions, no additional cost. Just raw values and message pack encoder / decoder. It is fast as well optimized v8 (in many years) and outperforms with event handling.
![https://imgur.com/gbq0th5](https://i.imgur.com/gbq0th5.png)

The runtime exposes small set of API to work with FiveM functions and expect some exports from scripts.

## API

More details can be found in [the implementation](https://github.com/ZOTTCE/fivem-wasm/blob/master/runtime/src/lib.rs) of runtime and [bindings for Rust](https://github.com/ZOTTCE/fivem-wasm/tree/master/bindings/core).

### Host

Module name: `host`

- `log` - logs a message in the console
- `invoke` - invokes native function
- `canonicalize_ref` - performs name transformation
- `invoke_ref_func` - calls ref functions

### Script

- `_start` - entry point (default WASM name)
- `__cfx_on_event` - callback for incoming events
- `__cfx_on_tick` - runtime calls scripts on each tick
- `__cfx_call_ref` - another script wants to call execute our function
- `__cfx_duplicate_ref`
- `__cfx_remove_ref`
- `__cfx_alloc` - runtime allocates some memory in script memory
- `__cfx_dealloc` - deallocate that has been allocated my us

## About the runtime implementation

So, why Rust if there is already C bindings for `wasmtime`?

- I have experience with it. For me it's best to write in Rust than make a billion mistakes in C/C++
- We can choose any WASM intepreter that is available for Rust (`wasmtime`, `wasmer`, `wasmi`)
- FiveM has a Rust crate and builds it
- Cargo makes working with dependencies easier

In the end we can simply ship prebuilt static libraries to remove Rust compiler as it would be if use `wasmtime` bindings with its libraries.

## Future work

- Split the runtime crate from bindings
- Write more bindings (or even framework-like library) in Rust (I want to support it). I haven't done much because don't want to spend time on this project if it will not be merged
- Decide to allow or disallow access to filesystem from scripts (we can select directories)
- Include a crate in build process (I am not familiar with it)
- Make some changes in component code to make sure it is idiomatic
