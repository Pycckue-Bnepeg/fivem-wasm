## API
### host
module name: `host`

exports:
* `log(ptr: u32, len: u32)`
* `invoke(hash_hi: u32, hash_lo: u32, ptr: u32, len: u32, rettype: i32)`
    * `hash_hi, hash_lo` - 64bit hash of native function divided in two parts.
    * `ptr, len` - a pointer to an array of pointers to arguments

### wasm script
exports:
* `_start`
* `__alloc(size: u32, alignment: u32) -> u32`
* `__free(ptr: *const c_void, size: u32, alignment: u32)`
* `on_tick()`
* `on_event(event_name: *const u8, args: *const u8, args_len: u32, src: *const u8)`
    * `event_name` - C string
    * `args` - an array of msgpack bytes
    * `src` - C string of the event source
