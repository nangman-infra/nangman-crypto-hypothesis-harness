use crate::args::S3OutputArgs;
use crate::model::{
    HARNESS_REPORT_SCHEMA_VERSION, HARNESS_RESULT_SCHEMA_VERSION, HarnessResult, HarnessRunReport,
    RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION, ResearchInputManifest,
};
use crate::time::partition;
use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::storage::{ObjectStore, ObjectStoreConfig};
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub(crate) fn write_outputs_to_dir(
    output_dir: &Path,
    created_at_ms: i64,
    results: &[HarnessResult],
    report: &HarnessRunReport,
) -> AppResult<Vec<String>> {
    let mut written = Vec::new();
    let result_key = harness_result_key(created_at_ms, &report.harness_run_report_id);
    written.push(write_jsonl(output_dir, &result_key, results)?);
    let report_key = harness_report_key(created_at_ms, &report.harness_run_report_id);
    written.push(write_json(output_dir, &report_key, report)?);
    Ok(written)
}

pub(crate) async fn write_outputs_to_s3(
    s3: &S3OutputArgs,
    created_at_ms: i64,
    results: &[HarnessResult],
    report: &HarnessRunReport,
) -> AppResult<Vec<String>> {
    let store = ObjectStore::connect(ObjectStoreConfig {
        endpoint: s3.endpoint.clone(),
        bucket: s3.bucket.clone(),
        region: s3.region.clone(),
        force_path_style: s3.force_path_style,
        profile: s3.profile.clone(),
        access_key_id: None,
        secret_access_key: None,
    })
    .await?;
    let result_key = prefixed_key(
        &s3.prefix,
        &harness_result_key(created_at_ms, &report.harness_run_report_id),
    );
    let report_key = prefixed_key(
        &s3.prefix,
        &harness_report_key(created_at_ms, &report.harness_run_report_id),
    );
    store
        .put_bytes_idempotent(&result_key, jsonl_bytes(results)?, "application/x-ndjson")
        .await?;
    store
        .put_bytes_idempotent(
            &report_key,
            serde_json::to_vec_pretty(report)?,
            "application/json",
        )
        .await?;
    Ok(vec![
        format!("s3://{}/{}", s3.bucket, result_key),
        format!("s3://{}/{}", s3.bucket, report_key),
    ])
}

pub(crate) async fn write_research_manifest_to_s3(
    s3: &S3OutputArgs,
    created_at_ms: i64,
    manifest: &ResearchInputManifest,
) -> AppResult<String> {
    let store = ObjectStore::connect(ObjectStoreConfig {
        endpoint: s3.endpoint.clone(),
        bucket: s3.bucket.clone(),
        region: s3.region.clone(),
        force_path_style: s3.force_path_style,
        profile: s3.profile.clone(),
        access_key_id: None,
        secret_access_key: None,
    })
    .await?;
    let packet_id = manifest
        .research_packet_id
        .as_deref()
        .ok_or_else(|| AppError::validation("research manifest packet id is required"))?;
    let key = prefixed_key(&s3.prefix, &research_manifest_key(created_at_ms, packet_id));
    let bytes = serde_json::to_vec_pretty(manifest)?;
    match store
        .put_bytes_idempotent(&key, bytes, "application/json")
        .await
    {
        Ok(()) => {}
        Err(AppError::Validation(message)) if message.contains("idempotency conflict") => {}
        Err(error) => return Err(error),
    }
    Ok(format!("s3://{}/{}", s3.bucket, key))
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
    let key = format!(
        "research-input-manifest/{}",
        research_manifest_key(created_at_ms, packet_id)
    );
    write_json(output_dir, &key, manifest)
}

pub(crate) fn harness_result_key(created_at_ms: i64, report_id: &str) -> String {
    let part = partition(created_at_ms);
    format!(
        "hypothesis-harness-result/schema={}/dt={}/hour={:02}/harness_run_report_id={}/part-000001.jsonl",
        HARNESS_RESULT_SCHEMA_VERSION, part.date, part.hour, report_id
    )
}

fn harness_report_key(created_at_ms: i64, report_id: &str) -> String {
    let part = partition(created_at_ms);
    format!(
        "hypothesis-harness-report/schema={}/dt={}/hour={:02}/harness_run_report_id={}/report.json",
        HARNESS_REPORT_SCHEMA_VERSION, part.date, part.hour, report_id
    )
}

fn research_manifest_key(_created_at_ms: i64, packet_id: &str) -> String {
    format!(
        "schema={}/dedupe_key={}/manifest.json",
        RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION, packet_id
    )
}

fn write_jsonl<T: Serialize>(output_dir: &Path, key: &str, records: &[T]) -> AppResult<String> {
    let bytes = jsonl_bytes(records)?;
    write_bytes(output_dir, key, &bytes)
}

fn write_json<T: Serialize>(output_dir: &Path, key: &str, record: &T) -> AppResult<String> {
    write_bytes(output_dir, key, &serde_json::to_vec_pretty(record)?)
}

fn write_bytes(output_dir: &Path, key: &str, bytes: &[u8]) -> AppResult<String> {
    let path = output_dir.join(key);
    let parent = path.parent().ok_or_else(|| {
        AppError::validation(format!("output path has no parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent)?;
    let mut file = File::create(&path)?;
    file.write_all(bytes)?;
    Ok(path.display().to_string())
}

fn jsonl_bytes<T: Serialize>(records: &[T]) -> AppResult<Vec<u8>> {
    let mut bytes = Vec::new();
    for record in records {
        serde_json::to_writer(&mut bytes, record)?;
        bytes.push(b'\n');
    }
    Ok(bytes)
}

fn prefixed_key(prefix: &str, key: &str) -> String {
    let prefix = prefix.trim_matches('/');
    if prefix.is_empty() {
        key.to_owned()
    } else {
        format!("{prefix}/{key}")
    }
}
