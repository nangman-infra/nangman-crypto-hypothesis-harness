use super::pressure::{harness_pressure_signature, has_harness_pressure};
use super::refs::{market_refs_for_type, ref_signature, s3_uri_for_candidate_bundle};
use super::selection::selected_candidate_bundles;
use crate::args::Args;
use crate::hash::stable_id;
use crate::model::{
    HarnessResult, RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION, ResearchArtifactRef,
    ResearchInputManifest, ResearchRuntimeBudgetPolicy,
};
use intel_candidate_app::model::IntelCandidateEvidenceBundle;

pub(crate) fn build_research_input_manifest(
    args: &Args,
    _created_at_ms: i64,
    _report_id: &str,
    harness_result_s3_uri: Option<&str>,
    results: &[HarnessResult],
    bundles: &[IntelCandidateEvidenceBundle],
    historical_replay_run_index_refs: &[ResearchArtifactRef],
) -> Option<ResearchInputManifest> {
    if !args.promotion_gate_enabled || !has_harness_pressure(args, results) {
        return None;
    }
    let candidate_bundle_s3 = args.candidate_bundle_s3.as_ref()?;
    let selected = selected_candidate_bundles(args, bundles);
    if selected.is_empty() {
        return None;
    }

    let candidate_bundle_refs = selected
        .iter()
        .map(|bundle| ResearchArtifactRef {
            uri: s3_uri_for_candidate_bundle(&candidate_bundle_s3.bucket, bundle),
        })
        .collect::<Vec<_>>();
    let market_feature_delta_refs = market_refs_for_type(
        args,
        &selected,
        "market_feature_delta",
        args.promotion_gate_max_market_refs,
    );
    let market_regime_context_refs = market_refs_for_type(
        args,
        &selected,
        "market_regime_context",
        args.promotion_gate_max_market_refs,
    );
    let hypothesis_harness_result_refs = harness_result_s3_uri
        .map(|uri| {
            vec![ResearchArtifactRef {
                uri: uri.to_owned(),
            }]
        })
        .unwrap_or_default()
        .into_iter()
        .take(args.promotion_gate_max_harness_result_refs)
        .collect::<Vec<_>>();
    let candidate_keys = selected
        .iter()
        .map(|bundle| bundle.candidate_lifecycle_key.as_str())
        .collect::<Vec<_>>();
    let pressure_signature = harness_pressure_signature(args, results);
    let research_packet_id = stable_id(
        "research_manifest",
        &[
            &candidate_keys.join("|"),
            &ref_signature(&market_feature_delta_refs),
            &ref_signature(&market_regime_context_refs),
            &pressure_signature,
        ],
    );

    Some(ResearchInputManifest {
        schema_version: RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION.to_owned(),
        research_packet_id: Some(research_packet_id),
        run_scope: Some("hypothesis_harness_p2_retest".to_owned()),
        candidate_bundle_refs,
        market_feature_delta_refs,
        market_regime_context_refs,
        hypothesis_harness_result_refs,
        historical_replay_run_refs: Vec::new(),
        historical_replay_run_index_refs: historical_replay_run_index_refs.to_vec(),
        runtime_budget_policy: ResearchRuntimeBudgetPolicy {
            max_candidate_bundle_count: args.promotion_gate_max_candidates,
            max_market_artifact_ref_count: args.promotion_gate_max_market_refs,
            max_hypothesis_harness_result_ref_count: args.promotion_gate_max_harness_result_refs,
            max_historical_replay_run_ref_count: historical_replay_run_index_refs
                .len()
                .max(args.historical_replay_run_index_s3_read_limit)
                .max(1),
            max_replay_run_count: args.promotion_gate_max_replay_runs,
        },
    })
}
