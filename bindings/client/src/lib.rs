use fivem_core::invoker::{invoke, Val};
use serde::Serialize;

pub fn emit_net<T: Serialize>(event_name: &str, payload: T) {
    if let Ok(payload) = rmp_serde::to_vec(&payload) {
        let args = &[
            Val::String(event_name),
            Val::Bytes(&payload),
            Val::Integer(payload.len() as _),
        ];

        let _ = invoke::<(), _>(0x7FDD1128, args); // TRIGGER_SERVER_EVENT_INTERNAL
    }
}
