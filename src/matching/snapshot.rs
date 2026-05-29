use std::collections::{BTreeMap, BTreeSet};

use intel_candidate_app::model::{IntelCandidateHypothesisState, MarketFeatureDelta};

use super::baseline::{artifact_baseline, is_newer_than_baseline};
use super::metrics::{has_usable_change, metric_allowed_for_state};
use super::quality::is_usable_market_artifact_quality;
use super::symbols::canonical_symbol_candidates;
use super::types::{MatchStatus, MatchedMarketArtifacts};

pub(super) fn select_latest_matching_snapshot<'a>(
    state: &IntelCandidateHypothesisState,
    deltas: &'a [MarketFeatureDelta],
) -> MatchedMarketArtifacts<'a> {
    let symbols = state
        .normalized_symbols
        .iter()
        .flat_map(|symbol| canonical_symbol_candidates(symbol))
        .collect::<BTreeSet<_>>();
    let metric_allowed = metric_allowed_for_state(state);
    let symbol_matches = deltas
        .iter()
        .filter(|delta| {
            symbols.contains(&delta.symbol_canonical.to_ascii_uppercase())
                && metric_allowed(delta.metric_name.as_str())
                && is_usable_market_artifact_quality(&delta.quality_status)
                && has_usable_change(delta)
        })
        .collect::<Vec<_>>();
    if symbol_matches.is_empty() {
        return MatchedMarketArtifacts {
            deltas: Vec::new(),
            status: MatchStatus::NoSymbolMatch,
        };
    }
    let baseline = artifact_baseline(state);
    let fresh_matches = symbol_matches
        .into_iter()
        .filter(|delta| is_newer_than_baseline(delta, &baseline))
        .collect::<Vec<_>>();
    if fresh_matches.is_empty() {
        return MatchedMarketArtifacts {
            deltas: Vec::new(),
            status: MatchStatus::NoFreshMatch,
        };
    }
    let Some(latest_run_id) = fresh_matches
        .iter()
        .max_by_key(|delta| {
            (
                delta.known_as_of_ms,
                delta.window_end_ms,
                delta.l1_run_id.as_str(),
            )
        })
        .map(|delta| delta.l1_run_id.as_str())
    else {
        return MatchedMarketArtifacts {
            deltas: Vec::new(),
            status: MatchStatus::NoFreshMatch,
        };
    };
    let latest_snapshot = collapse_snapshot_to_latest_metrics(
        fresh_matches
            .into_iter()
            .filter(|delta| delta.l1_run_id == latest_run_id)
            .collect(),
    );
    MatchedMarketArtifacts {
        deltas: latest_snapshot,
        status: MatchStatus::Matched,
    }
}

fn collapse_snapshot_to_latest_metrics(
    snapshot: Vec<&MarketFeatureDelta>,
) -> Vec<&MarketFeatureDelta> {
    let mut latest_by_metric =
        BTreeMap::<(String, String, String, String), &MarketFeatureDelta>::new();
    for delta in snapshot {
        let key = (
            delta.venue.clone(),
            delta.symbol_canonical.clone(),
            delta.market_type.clone(),
            delta.metric_name.clone(),
        );
        let should_replace = latest_by_metric
            .get(&key)
            .map(|current| {
                (delta.window_end_ms, delta.known_as_of_ms)
                    > (current.window_end_ms, current.known_as_of_ms)
            })
            .unwrap_or(true);
        if should_replace {
            latest_by_metric.insert(key, delta);
        }
    }
    let mut collapsed = latest_by_metric.into_values().collect::<Vec<_>>();
    collapsed.sort_by(|left, right| {
        left.metric_name
            .cmp(&right.metric_name)
            .then_with(|| left.venue.cmp(&right.venue))
            .then_with(|| left.market_type.cmp(&right.market_type))
            .then_with(|| left.window_end_ms.cmp(&right.window_end_ms))
    });
    collapsed
}
