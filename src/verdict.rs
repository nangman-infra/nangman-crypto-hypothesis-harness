use crate::args::Args;
use crate::matching::MatchStatus;
use intel_candidate_app::model::IntelCandidateHypothesisState;

pub(crate) fn decide_verdict(
    state: &IntelCandidateHypothesisState,
    max_change: Option<f64>,
    match_status: MatchStatus,
    args: &Args,
) -> (String, String, Option<String>) {
    if !state.terminal_reasons.is_empty() && state.retryable_reasons.is_empty() {
        return (
            "PRUNE".to_owned(),
            "archive_until_revision".to_owned(),
            Some(state.terminal_reasons.join(",")),
        );
    }
    let Some(max_change) = max_change else {
        let failure_reason = match match_status {
            MatchStatus::NoFreshMatch => "no_new_market_feature_delta_since_hypothesis_state",
            MatchStatus::NoSymbolMatch | MatchStatus::Matched => "no_matching_market_feature_delta",
        };
        return (
            "OBSERVE".to_owned(),
            "wait_for_market_feature_delta".to_owned(),
            Some(failure_reason.to_owned()),
        );
    };
    if matches!(match_status, MatchStatus::Matched)
        && state.current_score >= args.promote_score_threshold
        && max_change >= args.promote_abs_delta_threshold_pct
    {
        return (
            "PROMOTE".to_owned(),
            "rescore_for_candidate_evidence_bundle".to_owned(),
            None,
        );
    }
    if matches!(match_status, MatchStatus::Matched)
        && max_change >= args.retest_abs_delta_threshold_pct
    {
        return (
            "RETEST".to_owned(),
            "run_next_window_harness".to_owned(),
            None,
        );
    }
    (
        "OBSERVE".to_owned(),
        "wait_for_stronger_delta_or_policy_change".to_owned(),
        Some("market_delta_below_retest_threshold".to_owned()),
    )
}
