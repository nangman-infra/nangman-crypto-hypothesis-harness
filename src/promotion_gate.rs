use crate::args::Args;
use crate::hash::stable_id;
use crate::model::{
    HarnessResult, RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION, ResearchArtifactRef,
    ResearchInputManifest, ResearchRuntimeBudgetPolicy,
};
use intel_candidate_app::model::IntelCandidateEvidenceBundle;
use std::collections::BTreeSet;

pub(crate) fn build_research_input_manifest(
    args: &Args,
    created_at_ms: i64,
    report_id: &str,
    harness_result_s3_uri: Option<&str>,
    results: &[HarnessResult],
    bundles: &[IntelCandidateEvidenceBundle],
    historical_replay_run_index_refs: &[ResearchArtifactRef],
) -> Option<ResearchInputManifest> {
    if !args.promotion_gate_enabled || !has_harness_pressure(args, results) {
        return None;
    }
    let selected = selected_candidate_bundles(args, bundles);
    if selected.is_empty() {
        return None;
    }

    let candidate_bundle_refs = selected
        .iter()
        .map(|bundle| ResearchArtifactRef {
            uri: s3_uri_for_candidate_bundle(args, bundle),
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
    let research_packet_id = stable_id(
        "research_manifest",
        &[
            report_id,
            &created_at_ms.to_string(),
            &candidate_keys.join("|"),
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
            max_historical_replay_run_ref_count: historical_replay_run_index_refs.len().max(1),
            max_replay_run_count: args.promotion_gate_max_replay_runs,
        },
    })
}

pub(crate) fn eligible_candidate_lifecycle_keys(
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
) -> BTreeSet<String> {
    selected_candidate_bundles(args, bundles)
        .into_iter()
        .map(|bundle| bundle.candidate_lifecycle_key.clone())
        .collect()
}

fn has_harness_pressure(args: &Args, results: &[HarnessResult]) -> bool {
    results.iter().any(|result| {
        result.verdict == "PROMOTE"
            || (args.promotion_gate_include_retest && result.verdict == "RETEST")
    })
}

fn selected_candidate_bundles<'a>(
    args: &Args,
    bundles: &'a [IntelCandidateEvidenceBundle],
) -> Vec<&'a IntelCandidateEvidenceBundle> {
    let mut selected = bundles
        .iter()
        .filter(|bundle| bundle.research_eligible)
        .filter(|bundle| bundle.candidate_score >= args.promotion_gate_min_candidate_score)
        .filter(|bundle| !bundle.candidate_lifecycle_key.trim().is_empty())
        .filter(|bundle| !bundle.bundle_key.trim().is_empty())
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        right
            .candidate_score
            .cmp(&left.candidate_score)
            .then_with(|| right.created_at_ms.cmp(&left.created_at_ms))
            .then_with(|| {
                left.candidate_lifecycle_key
                    .cmp(&right.candidate_lifecycle_key)
            })
            .then_with(|| left.bundle_key.cmp(&right.bundle_key))
    });
    selected.dedup_by(|left, right| left.candidate_lifecycle_key == right.candidate_lifecycle_key);
    selected.truncate(args.promotion_gate_max_candidates);
    selected
}

fn market_refs_for_type(
    args: &Args,
    bundles: &[&IntelCandidateEvidenceBundle],
    artifact_type: &str,
    max_refs: usize,
) -> Vec<ResearchArtifactRef> {
    let mut refs = BTreeSet::new();
    for bundle in bundles {
        for artifact in &bundle.selected_market_artifacts {
            let Some(key) = artifact.artifact_key.as_deref() else {
                continue;
            };
            match artifact_type {
                "market_feature_delta" => {
                    if artifact.artifact_type == "market_feature_delta" {
                        refs.insert(ResearchArtifactRef {
                            uri: market_l1_uri(args, key),
                        });
                    } else if artifact.artifact_type == "market_feature_delta_summary"
                        && let Some(delta_key) = market_feature_delta_key_from_summary(key)
                    {
                        refs.insert(ResearchArtifactRef {
                            uri: market_l1_uri(args, &delta_key),
                        });
                    }
                }
                "market_regime_context" if artifact.artifact_type == "market_regime_context" => {
                    refs.insert(ResearchArtifactRef {
                        uri: market_l1_uri(args, key),
                    });
                }
                _ => {}
            }
        }
    }
    refs.into_iter().take(max_refs).collect()
}

fn s3_uri_for_candidate_bundle(args: &Args, bundle: &IntelCandidateEvidenceBundle) -> String {
    let bucket = args
        .candidate_bundle_s3
        .as_ref()
        .map(|s3| s3.bucket.as_str())
        .unwrap_or_default();
    format!("s3://{}/{}", bucket, bundle.bundle_key)
}

fn market_l1_uri(args: &Args, key: &str) -> String {
    let bucket = args
        .market_feature_delta_s3
        .as_ref()
        .or(args.market_feature_delta_summary_s3.as_ref())
        .map(|s3| s3.bucket.as_str())
        .unwrap_or("nangman-crypto-dev-market-ingest-l1-<account-suffix>");
    format!("s3://{bucket}/{key}")
}

fn market_feature_delta_key_from_summary(key: &str) -> Option<String> {
    let key = key.strip_prefix("market_feature_delta_summary/")?;
    let run_id = key.strip_suffix("/summary.json")?;
    Some(format!("market_feature_delta/{run_id}/delta.json"))
}
