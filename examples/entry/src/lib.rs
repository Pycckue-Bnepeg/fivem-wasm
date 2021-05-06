use fivem_bindings::log;
use std::ffi::CStr;

#[no_mangle]
pub unsafe extern "C" fn on_event(
    cstring: *const i8,
    args: *const u8,
    args_length: u32,
    source: *const i8,
) {
    let text = CStr::from_ptr(cstring).to_str().unwrap().to_owned();
    let args = Vec::from(std::slice::from_raw_parts(args, args_length as _));
    let source = CStr::from_ptr(source).to_str().unwrap().to_owned();

    let args = decode_args(&args);

    log(format!("event: {:?}", text));
    log(format!("source: {:?}", source));
    log(format!("args: {:?}", args));
}

fn decode_args(buffer: &[u8]) -> rmpv::Value {
    let mut bf = buffer;
    rmpv::decode::read_value(&mut bf).unwrap()
}

#[no_mangle]
pub extern "C" fn _start() {
    fivem_bindings::invoker::register_resource_as_event_handler("onServerResourceStart");
    log("I AM FUCKING STARTED !");
}
