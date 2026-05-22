use crate::model::MarketArtifactInputs;
use intel_candidate_app::model::{
    IntelCandidateHypothesisState, MarketFeatureDelta, SelectedMarketArtifactTrace,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MatchStatus {
    NoSymbolMatch,
    NoFreshMatch,
    Matched,
}

#[derive(Debug, Clone)]
pub(crate) struct MatchedMarketArtifacts<'a> {
    pub(crate) deltas: Vec<&'a MarketFeatureDelta>,
    pub(crate) status: MatchStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArtifactBaseline {
    window_end_ms: i64,
    known_as_of_ms: i64,
}

pub(crate) fn matching_deltas<'a>(
    state: &IntelCandidateHypothesisState,
    market_artifacts: &'a MarketArtifactInputs,
) -> MatchedMarketArtifacts<'a> {
    let summary_match = select_latest_matching_snapshot(state, &market_artifacts.summary_deltas);
    if matches!(summary_match.status, MatchStatus::Matched) {
        return summary_match;
    }
    let detail_match = select_latest_matching_snapshot(state, &market_artifacts.detail_deltas);
    if matches!(detail_match.status, MatchStatus::Matched) {
        return detail_match;
    }
    let status = if matches!(summary_match.status, MatchStatus::NoFreshMatch)
        || matches!(detail_match.status, MatchStatus::NoFreshMatch)
    {
        MatchStatus::NoFreshMatch
    } else {
        MatchStatus::NoSymbolMatch
    };
    MatchedMarketArtifacts {
        deltas: Vec::new(),
        status,
    }
}

pub(crate) fn max_abs_change(
    deltas: &[&MarketFeatureDelta],
    accessor: impl Fn(&MarketFeatureDelta) -> Option<f64>,
) -> Option<f64> {
    deltas
        .iter()
        .filter_map(|delta| {
            accessor(delta)
                .filter(|value| value.is_finite())
                .map(f64::abs)
        })
        .reduce(f64::max)
}

fn select_latest_matching_snapshot<'a>(
    state: &IntelCandidateHypothesisState,
    deltas: &'a [MarketFeatureDelta],
) -> MatchedMarketArtifacts<'a> {
    let symbols = state
        .normalized_symbols
        .iter()
        .flat_map(|symbol| canonical_symbol_candidates(symbol))
        .collect::<BTreeSet<_>>();
    let symbol_matches = deltas
        .iter()
        .filter(|delta| {
            symbols.contains(&delta.symbol_canonical.to_ascii_uppercase())
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
    let latest_run_id = fresh_matches
        .iter()
        .max_by_key(|delta| {
            (
                delta.known_as_of_ms,
                delta.window_end_ms,
                delta.l1_run_id.as_str(),
            )
        })
        .map(|delta| delta.l1_run_id.as_str())
        .expect("fresh matches are not empty");
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

fn artifact_baseline(state: &IntelCandidateHypothesisState) -> ArtifactBaseline {
    state
        .selected_market_artifacts
        .iter()
        .filter(|artifact| is_market_feature_artifact(artifact))
        .max_by_key(|artifact| (artifact.window_end_ms, artifact.known_as_of_ms))
        .map(|artifact| ArtifactBaseline {
            window_end_ms: artifact.window_end_ms,
            known_as_of_ms: artifact.known_as_of_ms,
        })
        .unwrap_or(ArtifactBaseline {
            window_end_ms: state.updated_at_ms,
            known_as_of_ms: state.updated_at_ms,
        })
}

fn is_market_feature_artifact(artifact: &SelectedMarketArtifactTrace) -> bool {
    matches!(
        artifact.artifact_type.trim(),
        "market_feature_delta" | "market_feature_delta_summary"
    )
}

fn is_newer_than_baseline(delta: &MarketFeatureDelta, baseline: &ArtifactBaseline) -> bool {
    (delta.window_end_ms, delta.known_as_of_ms) > (baseline.window_end_ms, baseline.known_as_of_ms)
}

fn has_usable_change(delta: &MarketFeatureDelta) -> bool {
    is_finite_option(delta.change_pct_15m) || is_finite_option(delta.change_pct_1h)
}

fn is_finite_option(value: Option<f64>) -> bool {
    value.is_some_and(f64::is_finite)
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

fn canonical_symbol_candidates(symbol: &str) -> Vec<String> {
    let upper = symbol.trim().to_ascii_uppercase();
    let mut values = BTreeSet::from([upper.clone()]);
    for suffix in ["USDT", "USDC", "USD", "BUSD", "BTC", "ETH"] {
        if upper.len() > suffix.len() && upper.ends_with(suffix) {
            values.insert(upper.trim_end_matches(suffix).to_owned());
        }
    }
    values.into_iter().collect()
}

fn is_usable_market_artifact_quality(status: &str) -> bool {
    matches!(
        status.trim().to_ascii_lowercase().as_str(),
        "" | "complete" | "partial" | "available"
    )
}
