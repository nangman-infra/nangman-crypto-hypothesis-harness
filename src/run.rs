use crate::args::Args;
use crate::hash::{checksum_json, stable_id};
use crate::io::{
    read_candidate_bundles, read_historical_replay_run_index_refs, read_hypothesis_states,
    read_market_artifacts,
};
use crate::matching::{matching_deltas, max_abs_change};
use crate::model::{
    HARNESS_RESULT_SCHEMA_VERSION, HarnessResult, MarketArtifactInputs, PRODUCER_APP,
    ResearchInputManifest, RunSummary,
};
use crate::output::{
    write_outputs_to_dir, write_outputs_to_s3, write_research_manifest_to_dir,
    write_research_manifest_to_s3,
};
use crate::promotion_gate::{build_research_input_manifest, eligible_candidate_lifecycle_keys};
use crate::report::build_report;
use crate::time::now_ms;
use crate::verdict::decide_verdict;
use intel_candidate_app::error::AppResult;
use intel_candidate_app::model::IntelCandidateHypothesisState;

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
    let results = states
        .iter()
        .map(|state| {
            build_harness_result(
                state,
                &market_artifacts,
                created_at_ms,
                (now_ms() - started_at).max(0),
                &args,
            )
        })
        .collect::<Vec<_>>();
    let report = build_report(
        created_at_ms,
        hypothesis_state_s3_keys_read,
        market_feature_delta_s3_keys_read,
        &results,
    );
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

pub(crate) async fn handle_research_manifest_outputs(
    args: &Args,
    created_at_ms: i64,
    manifest: &ResearchInputManifest,
    output_files: &mut Vec<String>,
    output_s3_uris: &mut Vec<String>,
) -> AppResult<usize> {
    let (output_path, output_uri) = write_research_manifest_outputs(
        args,
        created_at_ms,
        manifest,
        output_files,
        output_s3_uris,
    )
    .await?;
    if output_path.is_none() && output_uri.is_none() {
        return Ok(0);
    }
    log_event(
        "research_manifest_created",
        serde_json::json!({
            "research_packet_id": manifest.research_packet_id,
            "candidate_bundle_ref_count": manifest.candidate_bundle_refs.len(),
            "market_feature_delta_ref_count": manifest.market_feature_delta_refs.len(),
            "market_regime_context_ref_count": manifest.market_regime_context_refs.len(),
            "hypothesis_harness_result_ref_count": manifest.hypothesis_harness_result_refs.len(),
            "historical_replay_run_index_ref_count": manifest.historical_replay_run_index_refs.len(),
            "output_path": output_path,
            "output_uri": output_uri
        }),
    );
    Ok(1)
}

pub(crate) async fn write_research_manifest_outputs(
    args: &Args,
    created_at_ms: i64,
    manifest: &ResearchInputManifest,
    output_files: &mut Vec<String>,
    output_s3_uris: &mut Vec<String>,
) -> AppResult<(Option<String>, Option<String>)> {
    let mut output_path = None;
    let mut output_uri = None;
    if let Some(output_dir) = args.research_manifest_output_dir.as_deref() {
        let path = write_research_manifest_to_dir(output_dir, created_at_ms, manifest)?;
        output_files.push(path.clone());
        output_path = Some(path);
    }
    if let Some(s3) = args.research_manifest_s3.as_ref() {
        let uri = write_research_manifest_to_s3(s3, created_at_ms, manifest).await?;
        output_s3_uris.push(uri.clone());
        output_uri = Some(uri);
    }
    Ok((output_path, output_uri))
}

pub(crate) fn build_harness_result(
    state: &IntelCandidateHypothesisState,
    market_artifacts: &MarketArtifactInputs,
    created_at_ms: i64,
    duration_ms: i64,
    args: &Args,
) -> HarnessResult {
    let matched = matching_deltas(state, market_artifacts);
    let max_abs_change_pct_15m = max_abs_change(&matched.deltas, |delta| delta.change_pct_15m);
    let max_abs_change_pct_1h = max_abs_change(&matched.deltas, |delta| delta.change_pct_1h);
    let max_change = max_abs_change_pct_15m
        .into_iter()
        .chain(max_abs_change_pct_1h)
        .fold(None, |acc: Option<f64>, value| {
            Some(acc.map_or(value, |current| current.max(value)))
        });
    let known_as_of_ms = matched
        .deltas
        .iter()
        .map(|delta| delta.known_as_of_ms)
        .max()
        .unwrap_or(created_at_ms);
    let (verdict, next_action, failure_reason) =
        decide_verdict(state, max_change, matched.status, args);
    let matched_market_artifact_ids = matched
        .deltas
        .iter()
        .map(|delta| delta.feature_delta_id.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let matched_metric_names = matched
        .deltas
        .iter()
        .map(|delta| delta.metric_name.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut result = HarnessResult {
        harness_result_id: stable_id(
            "harness_result",
            &[
                &state.hypothesis_id,
                &state.updated_at_ms.to_string(),
                &created_at_ms.to_string(),
                verdict.as_str(),
            ],
        ),
        schema_version: HARNESS_RESULT_SCHEMA_VERSION.to_owned(),
        producer_app: PRODUCER_APP.to_owned(),
        producer_version: env!("CARGO_PKG_VERSION").to_owned(),
        created_at_ms,
        hypothesis_id: state.hypothesis_id.clone(),
        hypothesis_state_key: state.state_key.clone(),
        latest_screening_event_id: state.latest_screening_event_id.clone(),
        scoring_policy_version: state.scoring_policy_version.clone(),
        hypothesis_type: state.hypothesis_type.clone(),
        current_state: state.current_state.clone(),
        current_score: state.current_score,
        harness_type: state.harness_queue_hint.clone(),
        input_refs: state.lineage_refs.clone(),
        matched_market_artifact_ids,
        matched_metric_names,
        max_abs_change_pct_15m,
        max_abs_change_pct_1h,
        verdict,
        next_action,
        failure_reason,
        cost_estimate_units: 1,
        duration_ms,
        known_as_of_ms,
        checksum: String::new(),
    };
    result.checksum = checksum_json(&result);
    result
}

fn log_event(event: &str, payload: serde_json::Value) {
    eprintln!(
        "{}",
        serde_json::json!({
            "event": event,
            "producer_app": PRODUCER_APP,
            "timestamp_ms": now_ms(),
            "payload": payload
        })
    );
}
