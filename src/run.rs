use crate::args::Args;
use crate::io::{
    read_candidate_bundles, read_historical_replay_run_index_refs, read_hypothesis_states,
    read_market_artifacts,
};
use crate::model::RunSummary;
use crate::output::{write_outputs_to_dir, write_outputs_to_s3};
use crate::promotion_gate::{build_research_input_manifest, eligible_candidate_lifecycle_keys};
use crate::report::build_report;
use crate::time::now_ms;
use intel_candidate_app::error::AppResult;

mod logging;
mod manifest_output;
mod result;

use logging::log_event;
pub(crate) use manifest_output::handle_research_manifest_outputs;
pub(crate) use result::build_harness_result;

pub(crate) async fn async_run(args: Args) -> AppResult<RunSummary> {
    log_event(
        "harness_started",
        serde_json::json!({
            "file_state_input_configured": args.hypothesis_state_file.is_some(),
            "s3_state_input_configured": args.hypothesis_state_s3.is_some(),
            "file_market_delta_summary_input_configured": args.market_feature_delta_summary_file.is_some(),
            "s3_market_delta_summary_input_configured": args.market_feature_delta_summary_s3.is_some(),
            "s3_market_delta_summary_read_limit": args.market_feature_delta_summary_s3_read_limit,
            "file_market_delta_input_configured": args.market_feature_delta_file.is_some(),
            "s3_market_delta_input_configured": args.market_feature_delta_s3.is_some(),
            "s3_market_delta_read_limit": args.market_feature_delta_s3_read_limit,
            "promotion_gate_enabled": args.promotion_gate_enabled,
            "promotion_gate_include_retest": args.promotion_gate_include_retest,
            "s3_candidate_bundle_input_configured": args.candidate_bundle_s3.is_some(),
            "s3_candidate_bundle_read_limit": args.candidate_bundle_s3_read_limit,
            "s3_historical_replay_run_index_input_configured": args.historical_replay_run_index_s3.is_some(),
            "s3_historical_replay_run_index_read_limit": args.historical_replay_run_index_s3_read_limit,
            "s3_output_configured": args.output_s3.is_some(),
            "local_research_manifest_output_configured": args.research_manifest_output_dir.is_some(),
            "s3_research_manifest_output_configured": args.research_manifest_s3.is_some(),
            "allow_no_output": args.allow_no_output
        }),
    );
    let (states, hypothesis_state_s3_keys_read) = read_hypothesis_states(&args).await?;
    let (market_artifacts, market_feature_delta_s3_keys_read) =
        read_market_artifacts(&args).await?;
    let created_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let started_at = now_ms();
    let mut results = Vec::with_capacity(states.len());
    for state in &states {
        results.push(build_harness_result(
            state,
            &market_artifacts,
            created_at_ms,
            (now_ms() - started_at).max(0),
            &args,
        )?);
    }
    let report = build_report(
        created_at_ms,
        hypothesis_state_s3_keys_read,
        market_feature_delta_s3_keys_read,
        &results,
    )?;
    let (candidate_bundles, candidate_bundle_s3_keys_read) = if args.promotion_gate_enabled {
        read_candidate_bundles(&args).await?
    } else {
        (Vec::new(), 0)
    };
    let selected_candidate_lifecycle_keys =
        eligible_candidate_lifecycle_keys(&args, &candidate_bundles);
    let (historical_replay_run_index_refs, historical_replay_run_index_s3_keys_read) =
        if args.promotion_gate_enabled {
            read_historical_replay_run_index_refs(&args, &selected_candidate_lifecycle_keys).await?
        } else {
            (Vec::new(), 0)
        };

    let mut output_files = Vec::new();
    if let Some(output_dir) = args.output_dir.as_deref() {
        output_files.extend(write_outputs_to_dir(
            output_dir,
            created_at_ms,
            &results,
            &report,
        )?);
    }

    let mut output_s3_uris = Vec::new();
    let mut harness_result_s3_uri = None;
    if let Some(s3) = args.output_s3.as_ref() {
        let written = write_outputs_to_s3(s3, created_at_ms, &results, &report).await?;
        harness_result_s3_uri = written.first().cloned();
        output_s3_uris.extend(written);
    }

    let mut research_manifests_created = 0;
    if let Some(manifest) = build_research_input_manifest(
        &args,
        created_at_ms,
        &report.harness_run_report_id,
        harness_result_s3_uri.as_deref(),
        &results,
        &candidate_bundles,
        &historical_replay_run_index_refs,
    ) {
        research_manifests_created = handle_research_manifest_outputs(
            &args,
            created_at_ms,
            &manifest,
            &mut output_files,
            &mut output_s3_uris,
        )
        .await?;
    } else if args.promotion_gate_enabled {
        log_event(
            "research_manifest_skipped",
            serde_json::json!({
                "reason": "no_eligible_harness_pressure_or_candidate_bundle",
                "candidate_bundle_count": candidate_bundles.len()
            }),
        );
    }

    Ok(RunSummary {
        hypothesis_states_read: states.len(),
        hypothesis_state_s3_keys_read,
        market_feature_delta_s3_keys_read,
        candidate_bundle_s3_keys_read,
        historical_replay_run_index_s3_keys_read,
        harness_results_created: results.len(),
        report_created: true,
        research_manifests_created,
        output_files,
        output_s3_uris,
    })
}
