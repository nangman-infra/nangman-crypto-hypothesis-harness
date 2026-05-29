use intel_candidate_app::error::AppResult;
use serde::Serialize;

pub(super) fn jsonl_bytes<T: Serialize>(records: &[T]) -> AppResult<Vec<u8>> {
    let mut bytes = Vec::new();
    for record in records {
        serde_json::to_writer(&mut bytes, record)?;
        bytes.push(b'\n');
    }
    Ok(bytes)
}
