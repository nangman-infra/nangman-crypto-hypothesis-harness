use super::json::{read_json_array_or_jsonl, read_json_array_or_jsonl_bytes};
use super::s3::{connect_store, latest_payload_keys, read_s3_records};
use crate::args::Args;
use crate::model::{MarketArtifactInputs, ResearchArtifactRef};
use crate::summary::expand_market_feature_delta_summary;
use intel_candidate_app::error::AppResult;
use intel_candidate_app::model::{
    IntelCandidateEvidenceBundle, IntelCandidateHypothesisState, MarketFeatureDelta,
    MarketFeatureDeltaSummary,
};
use serde::Deserialize;
use std::collections::BTreeSet;

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

#[derive(Debug, Deserialize)]
struct ReplayRunIndexRecordForSelection {
    source_candidate_lifecycle_key: String,
}
