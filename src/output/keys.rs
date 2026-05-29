use crate::model::{
    HARNESS_REPORT_SCHEMA_VERSION, HARNESS_RESULT_SCHEMA_VERSION,
    RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION,
};
use crate::time::partition;
use intel_candidate_app::error::{AppError, AppResult};
use std::path::{Component, Path};

const MAX_S3_OBJECT_KEY_BYTES: usize = 1024;

pub(crate) fn harness_result_key(created_at_ms: i64, report_id: &str) -> String {
    let part = partition(created_at_ms);
    format!(
        "hypothesis-harness-result/schema={}/dt={}/hour={:02}/harness_run_report_id={}/part-000001.jsonl",
        HARNESS_RESULT_SCHEMA_VERSION, part.date, part.hour, report_id
    )
}

pub(super) fn harness_report_key(created_at_ms: i64, report_id: &str) -> String {
    let part = partition(created_at_ms);
    format!(
        "hypothesis-harness-report/schema={}/dt={}/hour={:02}/harness_run_report_id={}/report.json",
        HARNESS_REPORT_SCHEMA_VERSION, part.date, part.hour, report_id
    )
}

pub(super) fn research_manifest_key(_created_at_ms: i64, packet_id: &str) -> String {
    format!(
        "schema={}/dedupe_key={}/manifest.json",
        RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION, packet_id
    )
}

pub(super) fn research_manifest_local_key(created_at_ms: i64, packet_id: &str) -> String {
    format!(
        "research-input-manifest/{}",
        research_manifest_key(created_at_ms, packet_id)
    )
}

pub(super) fn prefixed_key(prefix: &str, key: &str) -> AppResult<String> {
    let prefix = prefix.trim_matches('/');
    validate_prefix(prefix)?;
    let key = if prefix.is_empty() {
        key.to_owned()
    } else {
        format!("{prefix}/{key}")
    };
    validate_output_key(&key)?;
    Ok(key)
}

pub(super) fn validate_key_component(value: &str, label: &'static str) -> AppResult<()> {
    if value.is_empty() {
        return Err(AppError::validation(format!("{label} is required")));
    }
    if value == "." || value == ".." {
        return Err(AppError::validation(format!(
            "{label} must not be a period-only path segment"
        )));
    }
    if value
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '/' | '\\'))
    {
        return Err(AppError::validation(format!(
            "{label} must not contain path separators or control characters"
        )));
    }
    Ok(())
}

pub(super) fn validate_output_key(key: &str) -> AppResult<()> {
    validate_relative_key(key, false)
}

fn validate_prefix(prefix: &str) -> AppResult<()> {
    validate_relative_key(prefix, true)
}

fn validate_relative_key(value: &str, allow_empty: bool) -> AppResult<()> {
    if value.is_empty() {
        if allow_empty {
            return Ok(());
        }
        return Err(AppError::validation("output key is required"));
    }
    if value.len() > MAX_S3_OBJECT_KEY_BYTES {
        return Err(AppError::validation(format!(
            "output key must be at most {MAX_S3_OBJECT_KEY_BYTES} bytes"
        )));
    }
    if value.chars().any(|ch| ch.is_control() || ch == '\\') {
        return Err(AppError::validation(
            "output key must not contain control characters or backslashes",
        ));
    }
    if value
        .split('/')
        .any(|segment| matches!(segment, "." | ".."))
    {
        return Err(AppError::validation(
            "output key must not contain period-only path segments",
        ));
    }
    for component in Path::new(value).components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir if allow_empty => {}
            Component::Prefix(_)
            | Component::RootDir
            | Component::CurDir
            | Component::ParentDir => {
                return Err(AppError::validation(
                    "output key must be a relative object key without path traversal",
                ));
            }
        }
    }
    Ok(())
}
