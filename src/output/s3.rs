use crate::args::S3OutputArgs;
use crate::model::{HarnessResult, HarnessRunReport, ResearchInputManifest};
use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::storage::{ObjectStore, ObjectStoreConfig};

use super::keys::{
    harness_report_key, harness_result_key, prefixed_key, research_manifest_key,
    validate_key_component,
};
use super::serialize::jsonl_bytes;

pub(crate) async fn write_outputs_to_s3(
    s3: &S3OutputArgs,
    created_at_ms: i64,
    results: &[HarnessResult],
    report: &HarnessRunReport,
) -> AppResult<Vec<String>> {
    validate_key_component(&report.harness_run_report_id, "harness run report id")?;
    let store = connect_store(s3).await?;
    let result_key = prefixed_key(
        &s3.prefix,
        &harness_result_key(created_at_ms, &report.harness_run_report_id),
    )?;
    let report_key = prefixed_key(
        &s3.prefix,
        &harness_report_key(created_at_ms, &report.harness_run_report_id),
    )?;
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
    let store = connect_store(s3).await?;
    let packet_id = manifest
        .research_packet_id
        .as_deref()
        .ok_or_else(|| AppError::validation("research manifest packet id is required"))?;
    validate_key_component(packet_id, "research manifest packet id")?;
    let key = prefixed_key(&s3.prefix, &research_manifest_key(created_at_ms, packet_id))?;
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

async fn connect_store(s3: &S3OutputArgs) -> AppResult<ObjectStore> {
    ObjectStore::connect(ObjectStoreConfig {
        bucket: s3.bucket.clone(),
        region: s3.region.clone(),
        profile: s3.profile.clone(),
        access_key_id: None,
        secret_access_key: None,
    })
    .await
}
