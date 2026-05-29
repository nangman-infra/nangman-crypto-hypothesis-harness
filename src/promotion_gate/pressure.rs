use crate::args::Args;
use crate::model::HarnessResult;

pub(super) fn harness_pressure_signature(args: &Args, results: &[HarnessResult]) -> String {
    let mut signatures = results
        .iter()
        .filter(|result| {
            result.verdict == "PROMOTE"
                || (args.promotion_gate_include_retest && result.verdict == "RETEST")
        })
        .map(|result| {
            format!(
                "{}:{}:{}:{}",
                result.hypothesis_state_key,
                result.verdict,
                result.matched_market_artifact_ids.join(","),
                result.matched_metric_names.join(",")
            )
        })
        .collect::<Vec<_>>();
    signatures.sort();
    signatures.join("|")
}

pub(super) fn has_harness_pressure(args: &Args, results: &[HarnessResult]) -> bool {
    results.iter().any(|result| {
        result.verdict == "PROMOTE"
            || (args.promotion_gate_include_retest && result.verdict == "RETEST")
    })
}
