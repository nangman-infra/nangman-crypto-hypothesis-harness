use crate::hash::{checksum_json, stable_id};
use crate::model::{
    HARNESS_REPORT_SCHEMA_VERSION, HarnessResult, HarnessRunReport, PRODUCER_APP, VerdictCount,
};
use crate::output::harness_result_key;
use intel_candidate_app::error::AppResult;

pub(crate) fn build_report(
    created_at_ms: i64,
    input_hypothesis_s3_keys_read: usize,
    input_market_delta_s3_keys_read: usize,
    results: &[HarnessResult],
) -> AppResult<HarnessRunReport> {
    let report_id = stable_id(
        "harness_report",
        &[&created_at_ms.to_string(), &results.len().to_string()],
    );
    let output_result_key = harness_result_key(created_at_ms, &report_id);
    let mut report = HarnessRunReport {
        harness_run_report_id: report_id,
        schema_version: HARNESS_REPORT_SCHEMA_VERSION.to_owned(),
        producer_app: PRODUCER_APP.to_owned(),
        producer_version: env!("CARGO_PKG_VERSION").to_owned(),
        created_at_ms,
        input_hypothesis_count: results.len(),
        input_hypothesis_s3_keys_read,
        input_market_delta_s3_keys_read,
        result_count: results.len(),
        promote_count: count_verdict(results, "PROMOTE"),
        retest_count: count_verdict(results, "RETEST"),
        observe_count: count_verdict(results, "OBSERVE"),
        prune_count: count_verdict(results, "PRUNE"),
        output_result_key,
        verdict_summary: ["PROMOTE", "RETEST", "OBSERVE", "PRUNE"]
            .into_iter()
            .map(|verdict| VerdictCount {
                verdict: verdict.to_owned(),
                count: count_verdict(results, verdict),
            })
            .collect(),
        checksum: String::new(),
    };
    report.checksum = checksum_json(&report)?;
    Ok(report)
}

fn count_verdict(results: &[HarnessResult], verdict: &str) -> usize {
    results
        .iter()
        .filter(|result| result.verdict == verdict)
        .count()
}
