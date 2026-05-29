use crate::args::Args;
use crate::model::ResearchArtifactRef;
use intel_candidate_app::model::IntelCandidateEvidenceBundle;
use std::collections::BTreeSet;

pub(super) fn ref_signature(refs: &[ResearchArtifactRef]) -> String {
    let mut refs = refs
        .iter()
        .map(|item| item.uri.as_str())
        .collect::<Vec<_>>();
    refs.sort();
    refs.join("|")
}

pub(super) fn market_refs_for_type(
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

pub(super) fn s3_uri_for_candidate_bundle(
    bucket: &str,
    bundle: &IntelCandidateEvidenceBundle,
) -> String {
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
