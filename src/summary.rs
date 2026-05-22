use crate::hash::stable_id;
use intel_candidate_app::model::{MarketFeatureDelta, MarketFeatureDeltaSummary};

pub(crate) fn expand_market_feature_delta_summary(
    summary: MarketFeatureDeltaSummary,
) -> Vec<MarketFeatureDelta> {
    let mut deltas = Vec::new();
    for row in summary.rows {
        for metric in row.metrics {
            deltas.push(MarketFeatureDelta {
                schema_version: summary.schema_version.clone(),
                feature_delta_id: stable_id(
                    "market_delta_summary_metric",
                    &[
                        &summary.feature_delta_summary_id,
                        &row.venue,
                        &row.symbol_native,
                        &row.symbol_canonical,
                        &row.market_type,
                        &metric.metric_name,
                        &metric.window_start_ms.to_string(),
                        &metric.window_end_ms.to_string(),
                    ],
                ),
                l1_run_id: summary.l1_run_id.clone(),
                metric_name: metric.metric_name,
                venue: row.venue.clone(),
                symbol_native: row.symbol_native.clone(),
                symbol_canonical: row.symbol_canonical.clone(),
                market_type: row.market_type.clone(),
                value_now: metric.value_now,
                value_15m_ago: metric.value_15m_ago,
                value_1h_ago: metric.value_1h_ago,
                change_pct_15m: metric.change_pct_15m,
                change_pct_1h: metric.change_pct_1h,
                price_change_same_window: metric.price_change_same_window,
                volume_change_same_window: metric.volume_change_same_window,
                oi_price_divergence: metric.oi_price_divergence,
                window_start_ms: metric.window_start_ms,
                window_end_ms: metric.window_end_ms,
                known_as_of_ms: row.known_as_of_ms,
                quality_status: if metric.quality_status.trim().is_empty() {
                    row.quality_status.clone()
                } else {
                    metric.quality_status
                },
                missing_reasons: row.missing_reasons.clone(),
            });
        }
    }
    deltas
}
