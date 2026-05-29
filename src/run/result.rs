use crate::args::Args;
use crate::hash::{checksum_json, stable_id};
use crate::matching::{matching_deltas, max_abs_change};
use crate::model::{
    HARNESS_RESULT_SCHEMA_VERSION, HarnessResult, MarketArtifactInputs, PRODUCER_APP,
};
use crate::verdict::decide_verdict;
use intel_candidate_app::error::AppResult;
use intel_candidate_app::model::IntelCandidateHypothesisState;
use std::collections::BTreeSet;

pub(crate) fn build_harness_result(
    state: &IntelCandidateHypothesisState,
    market_artifacts: &MarketArtifactInputs,
    created_at_ms: i64,
    duration_ms: i64,
    args: &Args,
) -> AppResult<HarnessResult> {
    let matched = matching_deltas(state, market_artifacts);
    let max_abs_change_pct_15m = max_abs_change(&matched.deltas, |delta| delta.change_pct_15m);
    let max_abs_change_pct_1h = max_abs_change(&matched.deltas, |delta| delta.change_pct_1h);
    let max_change = max_abs_change_pct_15m
        .into_iter()
        .chain(max_abs_change_pct_1h)
        .fold(None, |acc: Option<f64>, value| {
            Some(acc.map_or(value, |current| current.max(value)))
        });
    let known_as_of_ms = matched
        .deltas
        .iter()
        .map(|delta| delta.known_as_of_ms)
        .max()
        .unwrap_or(created_at_ms);
    let (verdict, next_action, failure_reason) =
        decide_verdict(state, max_change, matched.status, args);
    let matched_market_artifact_ids = matched
        .deltas
        .iter()
        .map(|delta| delta.feature_delta_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let matched_metric_names = matched
        .deltas
        .iter()
        .map(|delta| delta.metric_name.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut result = HarnessResult {
        harness_result_id: stable_id(
            "harness_result",
            &[
                &state.hypothesis_id,
                &state.updated_at_ms.to_string(),
                &created_at_ms.to_string(),
                verdict.as_str(),
            ],
        ),
        schema_version: HARNESS_RESULT_SCHEMA_VERSION.to_owned(),
        producer_app: PRODUCER_APP.to_owned(),
        producer_version: env!("CARGO_PKG_VERSION").to_owned(),
        created_at_ms,
        hypothesis_id: state.hypothesis_id.clone(),
        hypothesis_state_key: state.state_key.clone(),
        latest_screening_event_id: state.latest_screening_event_id.clone(),
        scoring_policy_version: state.scoring_policy_version.clone(),
        hypothesis_type: state.hypothesis_type.clone(),
        current_state: state.current_state.clone(),
        current_score: state.current_score,
        harness_type: state.harness_queue_hint.clone(),
        input_refs: state.lineage_refs.clone(),
        matched_market_artifact_ids,
        matched_metric_names,
        max_abs_change_pct_15m,
        max_abs_change_pct_1h,
        verdict,
        next_action,
        failure_reason,
        cost_estimate_units: 1,
        duration_ms,
        known_as_of_ms,
        checksum: String::new(),
    };
    result.checksum = checksum_json(&result)?;
    Ok(result)
}
