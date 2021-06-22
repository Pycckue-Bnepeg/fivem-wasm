pub mod events;
pub mod natives;
pub mod task;

pub fn emit_net<T: serde::Serialize>(event_name: &str, payload: T) {
    if let Ok(payload) = rmp_serde::to_vec(&payload) {
        natives::cfx::trigger_server_event_internal(
            event_name,
            payload.as_slice(),
            payload.len() as _,
        );
    }
}
