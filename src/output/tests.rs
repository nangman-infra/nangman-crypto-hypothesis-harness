use std::path::Path;

use crate::model::{
    RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION, ResearchArtifactRef, ResearchInputManifest,
    ResearchRuntimeBudgetPolicy,
};

use super::keys::{prefixed_key, validate_output_key};
use super::write_research_manifest_to_dir;

fn research_manifest(packet_id: &str) -> ResearchInputManifest {
    ResearchInputManifest {
        schema_version: RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION.to_owned(),
        research_packet_id: Some(packet_id.to_owned()),
        run_scope: Some("hypothesis_harness_p2_retest".to_owned()),
        candidate_bundle_refs: vec![ResearchArtifactRef {
            uri: "s3://candidate-bucket/candidate-evidence-bundle/schema=v1/part-000001.jsonl"
                .to_owned(),
        }],
        market_feature_delta_refs: Vec::new(),
        market_regime_context_refs: Vec::new(),
        hypothesis_harness_result_refs: Vec::new(),
        historical_replay_run_refs: Vec::new(),
        historical_replay_run_index_refs: Vec::new(),
        runtime_budget_policy: ResearchRuntimeBudgetPolicy::default(),
    }
}

#[test]
fn local_manifest_output_rejects_traversal_packet_id() {
    let output_dir = std::env::temp_dir().join(format!(
        "hypothesis-harness-output-traversal-test-{}",
        std::process::id()
    ));
    let error =
        write_research_manifest_to_dir(&output_dir, 7_200_000, &research_manifest("../escape"))
            .unwrap_err()
            .to_string();

    assert!(error.contains("research manifest packet id"));
    assert!(!output_dir.join("escape").exists());
}

#[test]
fn local_manifest_output_requires_absolute_output_dir() {
    let error = write_research_manifest_to_dir(
        Path::new("relative-output"),
        7_200_000,
        &research_manifest("research_packet_001"),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("absolute path"));
}

#[test]
fn output_key_validation_rejects_local_escape_shapes() {
    for key in [
        "/tmp/out.json",
        "research-input-manifest/../manifest.json",
        "research-input-manifest/./manifest.json",
        "research-input-manifest\\manifest.json",
        "research-input-manifest/\n/manifest.json",
    ] {
        let error = validate_output_key(key).unwrap_err().to_string();
        assert!(
            error.contains("output key"),
            "expected output key error for {key:?}, got {error}"
        );
    }
}

#[test]
fn prefixed_s3_key_rejects_traversal_prefix() {
    let error = prefixed_key("../escape", "schema=v1/manifest.json")
        .unwrap_err()
        .to_string();

    assert!(error.contains("output key"));
}

#[test]
fn prefixed_s3_key_preserves_normal_prefix_contract() {
    assert_eq!(
        prefixed_key(
            "/hypothesis-harness/research-input/",
            "schema=research_input_manifest_v1/dedupe_key=research_packet_001/manifest.json",
        )
        .unwrap(),
        "hypothesis-harness/research-input/schema=research_input_manifest_v1/dedupe_key=research_packet_001/manifest.json"
    );
}
