mod baseline;
mod metrics;
mod quality;
mod snapshot;
mod symbols;
mod types;

use crate::model::MarketArtifactInputs;
use intel_candidate_app::model::IntelCandidateHypothesisState;

pub(crate) use metrics::max_abs_change;
pub(crate) use types::{MatchStatus, MatchedMarketArtifacts};

use snapshot::select_latest_matching_snapshot;

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
