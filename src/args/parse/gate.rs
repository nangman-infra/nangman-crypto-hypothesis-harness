use intel_candidate_app::error::AppResult;

use super::super::types::Args;
use super::super::validation::{non_negative_f64, non_negative_i64, positive_usize};

pub(super) fn parse_gate_arg(
    arg: &str,
    values: &mut impl Iterator<Item = String>,
    args: &mut Args,
) -> AppResult<bool> {
    match arg {
        "--promotion-gate-enabled" => args.promotion_gate_enabled = true,
        "--promotion-gate-include-retest" => args.promotion_gate_include_retest = true,
        "--allow-no-output" => args.allow_no_output = true,
        "--promotion-gate-min-candidate-score" => {
            args.promotion_gate_min_candidate_score =
                non_negative_i64(values.next(), "--promotion-gate-min-candidate-score")?;
        }
        "--promotion-gate-max-candidates" => {
            args.promotion_gate_max_candidates =
                positive_usize(values.next(), "--promotion-gate-max-candidates")?;
        }
        "--promotion-gate-max-market-refs" => {
            args.promotion_gate_max_market_refs =
                positive_usize(values.next(), "--promotion-gate-max-market-refs")?;
        }
        "--promotion-gate-max-harness-result-refs" => {
            args.promotion_gate_max_harness_result_refs =
                positive_usize(values.next(), "--promotion-gate-max-harness-result-refs")?;
        }
        "--promotion-gate-max-replay-runs" => {
            args.promotion_gate_max_replay_runs =
                positive_usize(values.next(), "--promotion-gate-max-replay-runs")?;
        }
        "--now-ms" => args.now_ms = Some(non_negative_i64(values.next(), "--now-ms")?),
        "--promote-score-threshold" => {
            args.promote_score_threshold =
                non_negative_i64(values.next(), "--promote-score-threshold")?;
        }
        "--promote-abs-delta-threshold-pct" => {
            args.promote_abs_delta_threshold_pct =
                non_negative_f64(values.next(), "--promote-abs-delta-threshold-pct")?;
        }
        "--retest-abs-delta-threshold-pct" => {
            args.retest_abs_delta_threshold_pct =
                non_negative_f64(values.next(), "--retest-abs-delta-threshold-pct")?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}
