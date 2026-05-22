use crate::args::{Args, S3InputArgs};
use crate::model::{MarketArtifactInputs, ResearchArtifactRef};
use crate::summary::expand_market_feature_delta_summary;
use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::model::{
    IntelCandidateEvidenceBundle, IntelCandidateHypothesisState, MarketFeatureDelta,
    MarketFeatureDeltaSummary,
};
use intel_candidate_app::storage::{ObjectStore, ObjectStoreConfig};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) async fn read_hypothesis_states(
    args: &Args,
) -> AppResult<(Vec<IntelCandidateHypothesisState>, usize)> {
    let mut states = Vec::new();
    let mut s3_keys_read = 0;
    if let Some(path) = args.hypothesis_state_file.as_deref() {
        states.extend(read_json_array_or_jsonl::<IntelCandidateHypothesisState>(
            path,
        )?);
    }
    if let Some(s3) = args.hypothesis_state_s3.as_ref() {
        let (mut values, keys_read) =
            read_s3_records::<IntelCandidateHypothesisState>(s3, None).await?;
        states.append(&mut values);
        s3_keys_read += keys_read;
    }
    Ok((states, s3_keys_read))
}

pub(crate) async fn read_market_artifacts(args: &Args) -> AppResult<(MarketArtifactInputs, usize)> {
    let mut summary_deltas = Vec::new();
    let mut detail_deltas = Vec::new();
    let mut s3_keys_read = 0;
    if let Some(path) = args.market_feature_delta_summary_file.as_deref() {
        let summaries = read_json_array_or_jsonl::<MarketFeatureDeltaSummary>(path)?;
        summary_deltas.extend(
            summaries
                .into_iter()
                .flat_map(expand_market_feature_delta_summary),
        );
    }
    if let Some(s3) = args.market_feature_delta_summary_s3.as_ref() {
        let (summaries, keys_read) = read_s3_records::<MarketFeatureDeltaSummary>(
            s3,
            Some(args.market_feature_delta_summary_s3_read_limit),
        )
        .await?;
        summary_deltas.extend(
            summaries
                .into_iter()
                .flat_map(expand_market_feature_delta_summary),
        );
        s3_keys_read += keys_read;
    }
    if let Some(path) = args.market_feature_delta_file.as_deref() {
        detail_deltas.extend(read_json_array_or_jsonl::<MarketFeatureDelta>(path)?);
    }
    if let Some(s3) = args.market_feature_delta_s3.as_ref() {
        let (mut values, keys_read) = read_s3_records::<MarketFeatureDelta>(
            s3,
            Some(args.market_feature_delta_s3_read_limit),
        )
        .await?;
        detail_deltas.append(&mut values);
        s3_keys_read += keys_read;
    }
    Ok((
        MarketArtifactInputs {
            summary_deltas,
            detail_deltas,
        },
        s3_keys_read,
    ))
}

pub(crate) async fn read_candidate_bundles(
    args: &Args,
) -> AppResult<(Vec<IntelCandidateEvidenceBundle>, usize)> {
    let Some(s3) = args.candidate_bundle_s3.as_ref() else {
        return Ok((Vec::new(), 0));
    };
    read_s3_records::<IntelCandidateEvidenceBundle>(s3, Some(args.candidate_bundle_s3_read_limit))
        .await
}

pub(crate) async fn read_historical_replay_run_index_refs(
    args: &Args,
    selected_candidate_lifecycle_keys: &BTreeSet<String>,
) -> AppResult<(Vec<ResearchArtifactRef>, usize)> {
    let Some(s3) = args.historical_replay_run_index_s3.as_ref() else {
        return Ok((Vec::new(), 0));
    };
    if selected_candidate_lifecycle_keys.is_empty() {
        return Ok((Vec::new(), 0));
    }
    let store = connect_store(s3).await?;
    let mut keys = payload_keys(&store, s3).await?;
    keys.sort_unstable_by(|left, right| right.cmp(left));
    keys.truncate(args.historical_replay_run_index_s3_read_limit);

    let mut refs = Vec::new();
    let mut keys_read = 0;
    for key in keys {
        let bytes = store.get_bytes(&key).await?;
        let records = read_json_array_or_jsonl_bytes::<ReplayRunIndexRecordForSelection>(
            &format!("s3://{}/{}", s3.bucket, key),
            &bytes,
        )?;
        keys_read += 1;
        if records.iter().any(|record| {
            selected_candidate_lifecycle_keys.contains(&record.source_candidate_lifecycle_key)
        }) {
            refs.push(ResearchArtifactRef {
                uri: format!("s3://{}/{}", s3.bucket, key),
            });
        }
    }
    refs.sort();
    refs.dedup();
    Ok((refs, keys_read))
}

pub(crate) async fn read_s3_records<T: serde::de::DeserializeOwned>(
    s3: &S3InputArgs,
    latest_read_limit: Option<usize>,
) -> AppResult<(Vec<T>, usize)> {
    let store = connect_store(s3).await?;
    let mut records = Vec::new();
    let mut keys_read = 0;
    let mut keys = payload_keys(&store, s3).await?;
    if let Some(limit) = latest_read_limit {
        keys.sort_unstable_by(|left, right| right.cmp(left));
        keys.truncate(limit);
    }
    for key in keys {
        let bytes = store.get_bytes(&key).await?;
        records.extend(read_json_array_or_jsonl_bytes::<T>(
            &format!("s3://{}/{}", s3.bucket, key),
            &bytes,
        )?);
        keys_read += 1;
    }
    Ok((records, keys_read))
}

async fn connect_store(s3: &S3InputArgs) -> AppResult<ObjectStore> {
    ObjectStore::connect(ObjectStoreConfig {
        endpoint: s3.endpoint.clone(),
        bucket: s3.bucket.clone(),
        region: s3.region.clone(),
        force_path_style: s3.force_path_style,
        profile: s3.profile.clone(),
        access_key_id: None,
        secret_access_key: None,
    })
    .await
}

async fn payload_keys(store: &ObjectStore, s3: &S3InputArgs) -> AppResult<Vec<String>> {
    Ok(store
        .list_keys(&s3.prefix, s3.max_keys)
        .await?
        .into_iter()
        .filter(|key| is_json_payload_key(key))
        .collect())
}

#[derive(Debug, Deserialize)]
struct ReplayRunIndexRecordForSelection {
    source_candidate_lifecycle_key: String,
}

pub(crate) fn read_json_array_or_jsonl<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> AppResult<Vec<T>> {
    let bytes = fs::read(path)?;
    read_json_array_or_jsonl_bytes(&path.display().to_string(), &bytes)
}

pub(crate) fn read_json_array_or_jsonl_bytes<T: serde::de::DeserializeOwned>(
    label: &str,
    bytes: &[u8],
) -> AppResult<Vec<T>> {
    let text =
        std::str::from_utf8(bytes).map_err(|error| AppError::Json(format!("{label}: {error}")))?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(format!("{label} must not be empty")));
    }
    if trimmed.starts_with('[') {
        return Ok(serde_json::from_str(trimmed)?);
    }
    if trimmed.starts_with('{')
        && let Ok(value) = serde_json::from_str(trimmed)
    {
        return Ok(vec![value]);
    }
    let mut values = Vec::new();
    for (index, line) in trimmed.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        values.push(serde_json::from_str(line).map_err(|error| {
            AppError::Json(format!(
                "{label} line {} is not valid JSON: {error}",
                index + 1,
            ))
        })?);
    }
    Ok(values)
}

fn is_json_payload_key(key: &str) -> bool {
    key.ends_with(".json") || key.ends_with(".jsonl")
}
