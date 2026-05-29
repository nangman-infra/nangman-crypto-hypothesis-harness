use intel_candidate_app::model::{IntelCandidateHypothesisState, MarketFeatureDelta};

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

pub(super) fn has_usable_change(delta: &MarketFeatureDelta) -> bool {
    is_finite_option(delta.change_pct_15m) || is_finite_option(delta.change_pct_1h)
}

pub(super) fn metric_allowed_for_state(
    state: &IntelCandidateHypothesisState,
) -> impl Fn(&str) -> bool + '_ {
    let requires_derivatives_metric = state.harness_queue_hint == "derivatives_delta_persistence"
        || state.hypothesis_type == "derivatives_pressure_shift"
        || state
            .reasons
            .iter()
            .chain(state.retryable_reasons.iter())
            .any(|reason| reason == "derivatives_metric_delta_missing");
    move |metric_name| {
        !requires_derivatives_metric
            || matches!(
                metric_name,
                "open_interest" | "funding_rate" | "liquidation" | "long_short_ratio"
            )
    }
}

fn is_finite_option(value: Option<f64>) -> bool {
    value.is_some_and(f64::is_finite)
}
