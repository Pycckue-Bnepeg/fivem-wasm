## API
### host
module name: `host`

exports:
* `log(ptr: u32, len: u32)`
* `invoke(hash_hi: u32, hash_lo: u32, ptr: u32, len: u32)`
    * `hash_hi, hash_lo` - 64 битный хеш разбитый на части
    * `ptr, len` - аргументы для вызова нативной функции. указатель на массив указателей аргументов

### wasm
exports:
* `_start`
* `__alloc(size: u32, alignment: u32) -> u32`
* `__free(ptr: *const c_void, size: u32, alignment: u32)`
* `on_tick()`
* `on_event(event_name: *const u8, args: *const u8, args_len: u32, src: *const u8)`
    * `event_name` - си строка с именем события
    * `args` - массив длинной в `args_len` с аргументами, закодированные в `msgpack`
    * `src` - си строка с указанием кто вызвал событие
