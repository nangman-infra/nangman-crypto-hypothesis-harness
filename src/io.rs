use crate::args::{Args, S3InputArgs};
use crate::model::{MarketArtifactInputs, ResearchArtifactRef};
use crate::summary::expand_market_feature_delta_summary;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use aws_types::region::Region;
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

const S3_LIST_PAGE_SIZE: i32 = 1_000;
const S3_SCAN_LIMIT: usize = 100_000;

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
    let keys = latest_payload_keys(s3, args.historical_replay_run_index_s3_read_limit).await?;

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
    let keys = latest_payload_keys(s3, latest_read_limit.unwrap_or(s3.max_keys)).await?;
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
        bucket: s3.bucket.clone(),
        region: s3.region.clone(),
        profile: s3.profile.clone(),
        access_key_id: None,
        secret_access_key: None,
    })
    .await
}

async fn latest_payload_keys(s3: &S3InputArgs, limit: usize) -> AppResult<Vec<String>> {
    Ok(select_latest_payload_keys(
        list_payload_objects(s3).await?,
        limit,
    ))
}

async fn list_payload_objects(s3: &S3InputArgs) -> AppResult<Vec<ListedPayloadObject>> {
    let client = connect_s3_client(s3).await?;
    let mut objects = Vec::new();
    let mut continuation_token = None;
    loop {
        let mut request = client
            .list_objects_v2()
            .bucket(&s3.bucket)
            .prefix(&s3.prefix)
            .max_keys(S3_LIST_PAGE_SIZE);
        if let Some(token) = continuation_token {
            request = request.continuation_token(token);
        }
        let output = request.send().await.map_err(|error| {
            AppError::aws(format!(
                "list_objects_v2 bucket={} prefix={} error={error}",
                s3.bucket, s3.prefix
            ))
        })?;
        for object in output.contents() {
            let Some(key) = object.key() else {
                continue;
            };
            if !is_json_payload_key(key) {
                continue;
            }
            objects.push(ListedPayloadObject {
                key: key.to_owned(),
                last_modified_ms: object
                    .last_modified()
                    .and_then(|date_time| date_time.to_millis().ok())
                    .unwrap_or(0),
            });
            if objects.len() >= S3_SCAN_LIMIT {
                return Err(AppError::validation(format!(
                    "s3 prefix scan limit exceeded bucket={} prefix={} limit={S3_SCAN_LIMIT}; narrow the prefix",
                    s3.bucket, s3.prefix
                )));
            }
        }
        continuation_token = output.next_continuation_token().map(ToOwned::to_owned);
        if continuation_token.is_none() {
            break;
        }
    }
    Ok(objects)
}

async fn connect_s3_client(s3: &S3InputArgs) -> AppResult<Client> {
    let mut loader =
        aws_config::defaults(BehaviorVersion::latest()).region(Region::new(s3.region.clone()));
    if let Some(profile) = s3.profile.as_ref() {
        loader = loader.profile_name(profile);
    }
    let sdk_config = loader.load().await;
    Ok(Client::new(&sdk_config))
}

#[derive(Debug, Deserialize)]
struct ReplayRunIndexRecordForSelection {
    source_candidate_lifecycle_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ListedPayloadObject {
    pub(crate) key: String,
    pub(crate) last_modified_ms: i64,
}

pub(crate) fn select_latest_payload_keys(
    mut objects: Vec<ListedPayloadObject>,
    limit: usize,
) -> Vec<String> {
    objects.sort_unstable_by(|left, right| {
        right
            .last_modified_ms
            .cmp(&left.last_modified_ms)
            .then_with(|| right.key.cmp(&left.key))
    });
    objects
        .into_iter()
        .take(limit)
        .map(|object| object.key)
        .collect()
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
