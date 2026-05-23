use crate::args::{Args, parse_args};
use crate::model::{MarketArtifactInputs, ResearchArtifactRef};
use crate::promotion_gate::build_research_input_manifest;
use crate::run::build_harness_result;
use intel_candidate_app::model::{
    CandidateClass, IntelCandidateEvidenceBundle, IntelCandidateHypothesisState,
    MarketContextStatus, MarketFeatureDelta, ScoreBreakdown, SelectedMarketArtifactTrace,
};

fn state(score: i64) -> IntelCandidateHypothesisState {
    IntelCandidateHypothesisState {
        hypothesis_id: "hyp_001".to_owned(),
        state_key: "hypothesis-state/state.json".to_owned(),
        schema_version: "intel_candidate_hypothesis_state_v1".to_owned(),
        producer_app: "intel-candidate-app".to_owned(),
        producer_version: "0.1.0".to_owned(),
        created_at_ms: 1_000,
        updated_at_ms: 1_000,
        input_packet_id: "packet_001".to_owned(),
        input_packet_family_id: "family_001".to_owned(),
        input_packet_revision: 0,
        source_structured_packet_ids: vec!["packet_001".to_owned()],
        source_event_ids: vec!["source_001".to_owned()],
        supersedes_packet_id: None,
        supersedes_hypothesis_id: None,
        latest_screening_event_id: "screen_001".to_owned(),
        scoring_policy_version: "policy_v1".to_owned(),
        normalized_symbols: vec!["BTC".to_owned()],
        event_type: "funding_shift".to_owned(),
        hypothesis_type: "derivatives_pressure_shift".to_owned(),
        current_state: CandidateClass::WeakCandidate,
        current_score: score,
        previous_score: None,
        research_eligible: false,
        transition: "created_or_refreshed".to_owned(),
        next_action: "rerun_when_market_feature_delta_updates".to_owned(),
        reasons: vec!["derivatives_metric_delta_missing".to_owned()],
        retryable_reasons: vec!["derivatives_metric_delta_missing".to_owned()],
        terminal_reasons: Vec::new(),
        selected_market_artifacts: Vec::new(),
        market_context_ref: None,
        market_context_status: MarketContextStatus::StaleButUsable,
        evidence_quality_reasons: Vec::new(),
        score_breakdown: ScoreBreakdown {
            components: Vec::new(),
            final_score: score,
        },
        lineage_refs: vec!["packet_001".to_owned()],
        dirty_triggers: vec!["market_feature_delta_updated".to_owned()],
        harness_queue_hint: "derivatives_delta_persistence".to_owned(),
        idempotency_key: "idem_001".to_owned(),
        checksum: "checksum".to_owned(),
    }
}

fn build_args() -> Args {
    parse_args(
        [
            "--hypothesis-state-file",
            "/tmp/state.json",
            "--output-dir",
            "/tmp/out",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .unwrap()
}

fn build_promotion_args() -> Args {
    parse_args(
        [
            "--hypothesis-state-file",
            "/tmp/state.json",
            "--output-dir",
            "/tmp/out",
            "--market-feature-delta-s3-bucket",
            "market-bucket",
            "--market-feature-delta-s3-prefix",
            "market_feature_delta/",
            "--promotion-gate-enabled",
            "--promotion-gate-include-retest",
            "--candidate-bundle-s3-bucket",
            "candidate-bucket",
            "--candidate-bundle-s3-prefix",
            "candidate-evidence-bundle/priority=p2/",
            "--research-manifest-s3-bucket",
            "research-bucket",
            "--research-manifest-s3-prefix",
            "research-input-manifest/",
        ]
        .into_iter()
        .map(str::to_owned),
    )
    .unwrap()
}

fn state_with_market_baseline(
    score: i64,
    window_end_ms: i64,
    known_as_of_ms: i64,
) -> IntelCandidateHypothesisState {
    let mut state = state(score);
    state
        .selected_market_artifacts
        .push(SelectedMarketArtifactTrace {
            artifact_type: "market_feature_delta_summary".to_owned(),
            artifact_id: "summary_trace".to_owned(),
            artifact_key: Some(
                "market_feature_delta_summary/run_id=l1_prev/summary.json".to_owned(),
            ),
            l1_run_id: Some("l1_prev".to_owned()),
            symbol_canonical: Some("BTC".to_owned()),
            metric_name: Some("open_interest".to_owned()),
            scope: Some("symbol_metric".to_owned()),
            window_start_ms: window_end_ms - 1_000,
            window_end_ms,
            known_as_of_ms,
            quality_status: "complete".to_owned(),
        });
    state.updated_at_ms = known_as_of_ms;
    state
}

fn delta(
    feature_delta_id: &str,
    l1_run_id: &str,
    metric_name: &str,
    change_pct_15m: f64,
    change_pct_1h: f64,
    window_end_ms: i64,
    known_as_of_ms: i64,
) -> MarketFeatureDelta {
    MarketFeatureDelta {
        schema_version: "market_feature_delta_v1".to_owned(),
        feature_delta_id: feature_delta_id.to_owned(),
        l1_run_id: l1_run_id.to_owned(),
        metric_name: metric_name.to_owned(),
        venue: "binance".to_owned(),
        symbol_native: "BTCUSDT".to_owned(),
        symbol_canonical: "BTC".to_owned(),
        market_type: "perp".to_owned(),
        value_now: 100.0,
        value_15m_ago: Some(99.0),
        value_1h_ago: Some(90.0),
        change_pct_15m: Some(change_pct_15m),
        change_pct_1h: Some(change_pct_1h),
        price_change_same_window: Some(1.0),
        volume_change_same_window: Some(3.0),
        oi_price_divergence: Some(2.0),
        window_start_ms: window_end_ms.saturating_sub(1_000),
        window_end_ms,
        known_as_of_ms,
        quality_status: "complete".to_owned(),
        missing_reasons: Vec::new(),
    }
}

fn p2_bundle() -> IntelCandidateEvidenceBundle {
    serde_json::from_str(r#"{
        "candidate_id": "cand_001",
        "candidate_lifecycle_key": "cand_001:v1",
        "bundle_key": "candidate-evidence-bundle/priority=p2/schema=intel_candidate_evidence_bundle_v1/dt=2026-05-10/hour=01/candidate_id=cand_001/part-000001.jsonl",
        "producer_app": "intel-candidate-app",
        "producer_run_id": "run_001",
        "created_at_ms": 7200000,
        "event_time_ms": 1000,
        "published_at_ms": 1000,
        "fetched_at_ms": 1100,
        "structured_at_ms": 1200,
        "candidate_created_at_ms": 1300,
        "decision_available_at_ms": 1300,
        "forbidden_lookahead_boundary_ms": 1300,
        "schema_version": "intel_candidate_evidence_bundle_v1",
        "scoring_policy_version": "scoring-policy.v1",
        "normalized_symbols": ["BTC"],
        "input_packet_family_id": "family_001",
        "input_packet_revision": 0,
        "symbol_universe_snapshot_id": "universe_001",
        "universe_as_of_ms": 1200,
        "approved_universe_symbol": true,
        "event_types": ["funding_shift"],
        "hypothesis_type": "derivatives_pressure_shift",
        "allowed_horizons": ["1h"],
        "source_story_cluster_ids": ["cluster_001"],
        "source_structured_packet_ids": ["packet_001"],
        "source_context_flag_packet_ids": [],
        "evidence_refs": ["packet_001"],
        "text_evidence": [],
        "metric_evidence": [],
        "data_quality_summary": {
            "market_data_quality_summary_key": "market_data_quality_summary/run_id=l1_001/summary.json",
            "status": "available"
        },
        "selected_market_artifacts": [
            {
                "artifact_type": "market_feature_delta_summary",
                "artifact_id": "summary_001",
                "artifact_key": "market_feature_delta_summary/run_id=l1_001/summary.json",
                "l1_run_id": "l1_001",
                "symbol_canonical": "BTC",
                "metric_name": "open_interest",
                "scope": "symbol_metric",
                "window_start_ms": 1000,
                "window_end_ms": 2000,
                "known_as_of_ms": 2000,
                "quality_status": "complete"
            },
            {
                "artifact_type": "market_regime_context",
                "artifact_id": "regime_001",
                "artifact_key": "market_regime_context/run_id=l1_001/context.json",
                "l1_run_id": "l1_001",
                "symbol_canonical": "BTC",
                "metric_name": "regime",
                "scope": "symbol",
                "window_start_ms": 1000,
                "window_end_ms": 2000,
                "known_as_of_ms": 2000,
                "quality_status": "complete"
            }
        ],
        "candidate_class": "research_candidate",
        "candidate_score": 70,
        "score_breakdown": { "components": [], "final_score": 70 },
        "research_priority": "p2",
        "research_eligible": true,
        "validation_requirements": {
            "required_adapters": ["native_replay"],
            "optional_adapters": [],
            "min_unseen_windows": 1,
            "include_fee": true,
            "include_slippage": true,
            "include_latency_assumption": true,
            "include_liquidity_filter": true,
            "required_train_validation_split": true,
            "max_adapter_runtime_minutes": 15
        },
        "source_independence": {
            "source_event_count": 1,
            "independent_source_count": 1,
            "official_source_present": true,
            "duplicate_content_hashes": [],
            "original_source_ids": ["official"]
        },
        "symbol_resolution_trace": [],
        "confidence_summary": {},
        "contradiction_summary": [],
        "observe_or_reject_reasons": [],
        "parent_artifact_ids": ["packet_001"],
        "storage_uri": "s3://candidate-bucket/candidate-evidence-bundle/priority=p2/schema=intel_candidate_evidence_bundle_v1/dt=2026-05-10/hour=01/candidate_id=cand_001/part-000001.jsonl",
        "checksum": "checksum",
        "idempotency_key": "idem_001"
    }"#)
    .unwrap()
}

#[test]
fn promotes_only_when_score_and_delta_are_both_strong() {
    let args = build_args();
    let result = build_harness_result(
        &state(65),
        &MarketArtifactInputs {
            summary_deltas: vec![delta(
                "delta_001",
                "l1_001",
                "open_interest",
                1.0,
                6.0,
                2_000,
                2_000,
            )],
            detail_deltas: Vec::new(),
        },
        7_200_000,
        1,
        &args,
    );
    assert_eq!(result.verdict, "PROMOTE");
    assert_eq!(result.next_action, "rescore_for_candidate_evidence_bundle");
}

#[test]
fn observes_without_matching_market_delta() {
    let args = build_args();
    let result = build_harness_result(
        &state(65),
        &MarketArtifactInputs::default(),
        7_200_000,
        1,
        &args,
    );
    assert_eq!(result.verdict, "OBSERVE");
    assert_eq!(
        result.failure_reason.as_deref(),
        Some("no_matching_market_feature_delta")
    );
}

#[test]
fn observes_when_only_stale_market_delta_exists() {
    let args = build_args();
    let state = state_with_market_baseline(65, 5_000, 5_000);
    let result = build_harness_result(
        &state,
        &MarketArtifactInputs {
            summary_deltas: vec![delta(
                "delta_stale",
                "l1_001",
                "open_interest",
                1.0,
                9.0,
                5_000,
                5_000,
            )],
            detail_deltas: Vec::new(),
        },
        7_200_000,
        1,
        &args,
    );
    assert_eq!(result.verdict, "OBSERVE");
    assert_eq!(
        result.failure_reason.as_deref(),
        Some("no_new_market_feature_delta_since_hypothesis_state")
    );
}

#[test]
fn derivatives_harness_ignores_price_and_volume_only_delta() {
    let args = build_args();
    let result = build_harness_result(
        &state(65),
        &MarketArtifactInputs {
            summary_deltas: vec![
                delta("delta_price", "l1_001", "price", 10.0, 12.0, 2_000, 2_000),
                delta(
                    "delta_volume",
                    "l1_001",
                    "trade_volume",
                    25.0,
                    30.0,
                    2_000,
                    2_000,
                ),
            ],
            detail_deltas: Vec::new(),
        },
        7_200_000,
        1,
        &args,
    );
    assert_eq!(result.verdict, "OBSERVE");
    assert_eq!(
        result.failure_reason.as_deref(),
        Some("no_matching_market_feature_delta")
    );
    assert!(result.matched_metric_names.is_empty());
}

#[test]
fn non_derivatives_harness_can_retest_on_price_delta() {
    let args = build_args();
    let mut state = state(55);
    state.harness_queue_hint = "event_reaction_smoke".to_owned();
    state.hypothesis_type = "general_intel_observation".to_owned();
    state.reasons = vec!["missing_evidence".to_owned()];
    state.retryable_reasons.clear();

    let result = build_harness_result(
        &state,
        &MarketArtifactInputs {
            summary_deltas: vec![delta(
                "delta_price",
                "l1_001",
                "price",
                3.0,
                4.0,
                2_000,
                2_000,
            )],
            detail_deltas: Vec::new(),
        },
        7_200_000,
        1,
        &args,
    );
    assert_eq!(result.verdict, "RETEST");
    assert_eq!(result.matched_metric_names, vec!["price".to_owned()]);
}

#[test]
fn ignores_older_run_outlier_and_collapses_latest_metric_rows() {
    let args = build_args();
    let result = build_harness_result(
        &state(55),
        &MarketArtifactInputs {
            summary_deltas: Vec::new(),
            detail_deltas: vec![
                delta(
                    "delta_old_outlier",
                    "l1_old",
                    "open_interest",
                    1.0,
                    29_613_180.6,
                    5_000,
                    5_000,
                ),
                delta(
                    "delta_new_older_row",
                    "l1_new",
                    "open_interest",
                    1.0,
                    4.0,
                    5_900,
                    5_900,
                ),
                delta(
                    "delta_new_latest_row",
                    "l1_new",
                    "open_interest",
                    1.0,
                    6.0,
                    6_000,
                    6_000,
                ),
                delta(
                    "delta_new_funding",
                    "l1_new",
                    "funding_rate",
                    1.0,
                    3.0,
                    6_000,
                    6_000,
                ),
            ],
        },
        7_200_000,
        1,
        &args,
    );
    assert_eq!(result.verdict, "RETEST");
    assert_eq!(result.max_abs_change_pct_1h, Some(6.0));
    assert_eq!(
        result.matched_market_artifact_ids,
        vec![
            "delta_new_funding".to_owned(),
            "delta_new_latest_row".to_owned()
        ]
    );
    assert_eq!(
        result.matched_metric_names,
        vec!["funding_rate".to_owned(), "open_interest".to_owned()]
    );
}

#[test]
fn prefers_fresh_summary_over_detail_outlier() {
    let args = build_args();
    let result = build_harness_result(
        &state(55),
        &MarketArtifactInputs {
            summary_deltas: vec![delta(
                "summary_delta",
                "l1_summary",
                "open_interest",
                1.0,
                6.0,
                6_000,
                6_000,
            )],
            detail_deltas: vec![delta(
                "detail_outlier",
                "l1_detail",
                "open_interest",
                1.0,
                2_146_155.56,
                7_000,
                7_000,
            )],
        },
        7_200_000,
        1,
        &args,
    );
    assert_eq!(result.verdict, "RETEST");
    assert_eq!(result.max_abs_change_pct_1h, Some(6.0));
    assert_eq!(
        result.matched_market_artifact_ids,
        vec!["summary_delta".to_owned()]
    );
}

#[test]
fn builds_research_manifest_for_retest_p2_bundle() {
    let args = build_promotion_args();
    let result = build_harness_result(
        &state(55),
        &MarketArtifactInputs {
            summary_deltas: vec![delta(
                "summary_delta",
                "l1_summary",
                "open_interest",
                1.0,
                6.0,
                6_000,
                6_000,
            )],
            detail_deltas: Vec::new(),
        },
        7_200_000,
        1,
        &args,
    );
    assert_eq!(result.verdict, "RETEST");
    let manifest = build_research_input_manifest(
        &args,
        7_200_000,
        "report_001",
        Some("s3://research-bucket/hypothesis-harness/hypothesis-harness-result/schema=hypothesis_harness_result_v1/part-000001.jsonl"),
        &[result],
        &[p2_bundle()],
        &[ResearchArtifactRef {
            uri: "s3://research-bucket/replay-run-index/schema=replay_run_index_v1/dt=2026-05-10/hour=11/research_run_report_id=report_001/part-000001.jsonl".to_owned(),
        }],
    )
    .expect("manifest is created");
    assert_eq!(manifest.schema_version, "research_input_manifest_v1");
    assert_eq!(manifest.candidate_bundle_refs.len(), 1);
    assert_eq!(manifest.market_feature_delta_refs.len(), 1);
    assert_eq!(
        manifest.market_feature_delta_refs[0].uri,
        "s3://market-bucket/market_feature_delta/run_id=l1_001/delta.json"
    );
    assert_eq!(manifest.market_regime_context_refs.len(), 1);
    assert_eq!(manifest.hypothesis_harness_result_refs.len(), 1);
    assert_eq!(manifest.historical_replay_run_index_refs.len(), 1);
    assert_eq!(manifest.runtime_budget_policy.max_replay_run_count, 500);
}
