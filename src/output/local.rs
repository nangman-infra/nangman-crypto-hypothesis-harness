use crate::model::{HarnessResult, HarnessRunReport, ResearchInputManifest};
use intel_candidate_app::error::{AppError, AppResult};
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use super::keys::{
    harness_report_key, harness_result_key, research_manifest_local_key, validate_key_component,
    validate_output_key,
};
use super::serialize::jsonl_bytes;

pub(crate) fn write_outputs_to_dir(
    output_dir: &Path,
    created_at_ms: i64,
    results: &[HarnessResult],
    report: &HarnessRunReport,
) -> AppResult<Vec<String>> {
    validate_key_component(&report.harness_run_report_id, "harness run report id")?;
    let mut written = Vec::new();
    let result_key = harness_result_key(created_at_ms, &report.harness_run_report_id);
    written.push(write_jsonl(output_dir, &result_key, results)?);
    let report_key = harness_report_key(created_at_ms, &report.harness_run_report_id);
    written.push(write_json(output_dir, &report_key, report)?);
    Ok(written)
}

pub(crate) fn write_research_manifest_to_dir(
    output_dir: &Path,
    created_at_ms: i64,
    manifest: &ResearchInputManifest,
) -> AppResult<String> {
    let packet_id = manifest
        .research_packet_id
        .as_deref()
        .ok_or_else(|| AppError::validation("research manifest packet id is required"))?;
    validate_key_component(packet_id, "research manifest packet id")?;
    let key = research_manifest_local_key(created_at_ms, packet_id);
    write_json(output_dir, &key, manifest)
}

fn write_jsonl<T: Serialize>(output_dir: &Path, key: &str, records: &[T]) -> AppResult<String> {
    let bytes = jsonl_bytes(records)?;
    write_bytes(output_dir, key, &bytes)
}

fn write_json<T: Serialize>(output_dir: &Path, key: &str, record: &T) -> AppResult<String> {
    write_bytes(output_dir, key, &serde_json::to_vec_pretty(record)?)
}

fn write_bytes(output_dir: &Path, key: &str, bytes: &[u8]) -> AppResult<String> {
    if !output_dir.is_absolute() {
        return Err(AppError::validation(format!(
            "output dir must be an absolute path: {}",
            output_dir.display()
        )));
    }
    validate_output_key(key)?;
    let path = output_dir.join(key);
    let parent = path.parent().ok_or_else(|| {
        AppError::validation(format!("output path has no parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent)?;
    let mut file = File::create(&path)?;
    file.write_all(bytes)?;
    Ok(path.display().to_string())
}
