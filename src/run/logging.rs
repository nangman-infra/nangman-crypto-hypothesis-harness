use crate::model::PRODUCER_APP;
use crate::time::now_ms;

pub(super) fn log_event(event: &str, payload: serde_json::Value) {
    eprintln!(
        "{}",
        serde_json::json!({
            "event": event,
            "producer_app": PRODUCER_APP,
            "timestamp_ms": now_ms(),
            "payload": payload
        })
    );
}
