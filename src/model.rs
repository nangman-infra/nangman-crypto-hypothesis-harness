use intel_candidate_app::model::{CandidateClass, MarketFeatureDelta};
use serde::{Deserialize, Serialize};

pub(crate) const HARNESS_RESULT_SCHEMA_VERSION: &str = "hypothesis_harness_result_v1";
pub(crate) const HARNESS_REPORT_SCHEMA_VERSION: &str = "hypothesis_harness_report_v1";
pub(crate) const RESEARCH_INPUT_MANIFEST_SCHEMA_VERSION: &str = "research_input_manifest_v1";
pub(crate) const PRODUCER_APP: &str = "hypothesis-harness-app";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct HarnessResult {
    pub(crate) harness_result_id: String,
    pub(crate) schema_version: String,
    pub(crate) producer_app: String,
    pub(crate) producer_version: String,
    pub(crate) created_at_ms: i64,
    pub(crate) hypothesis_id: String,
    pub(crate) hypothesis_state_key: String,
    pub(crate) latest_screening_event_id: String,
    pub(crate) scoring_policy_version: String,
    pub(crate) hypothesis_type: String,
    pub(crate) current_state: CandidateClass,
    pub(crate) current_score: i64,
    pub(crate) harness_type: String,
    pub(crate) input_refs: Vec<String>,
    pub(crate) matched_market_artifact_ids: Vec<String>,
    pub(crate) matched_metric_names: Vec<String>,
    pub(crate) max_abs_change_pct_15m: Option<f64>,
    pub(crate) max_abs_change_pct_1h: Option<f64>,
    pub(crate) verdict: String,
    pub(crate) next_action: String,
    pub(crate) failure_reason: Option<String>,
    pub(crate) cost_estimate_units: u32,
    pub(crate) duration_ms: i64,
    pub(crate) known_as_of_ms: i64,
    pub(crate) checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct HarnessRunReport {
    pub(crate) harness_run_report_id: String,
    pub(crate) schema_version: String,
    pub(crate) producer_app: String,
    pub(crate) producer_version: String,
    pub(crate) created_at_ms: i64,
    pub(crate) input_hypothesis_count: usize,
    pub(crate) input_hypothesis_s3_keys_read: usize,
    pub(crate) input_market_delta_s3_keys_read: usize,
    pub(crate) result_count: usize,
    pub(crate) promote_count: usize,
    pub(crate) retest_count: usize,
    pub(crate) observe_count: usize,
    pub(crate) prune_count: usize,
    pub(crate) output_result_key: String,
    pub(crate) verdict_summary: Vec<VerdictCount>,
    pub(crate) checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct VerdictCount {
    pub(crate) verdict: String,
    pub(crate) count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RunSummary {
    pub(crate) hypothesis_states_read: usize,
    pub(crate) hypothesis_state_s3_keys_read: usize,
    pub(crate) market_feature_delta_s3_keys_read: usize,
    pub(crate) candidate_bundle_s3_keys_read: usize,
    pub(crate) historical_replay_run_index_s3_keys_read: usize,
    pub(crate) harness_results_created: usize,
    pub(crate) report_created: bool,
    pub(crate) research_manifests_created: usize,
    pub(crate) output_files: Vec<String>,
    pub(crate) output_s3_uris: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct MarketArtifactInputs {
    pub(crate) summary_deltas: Vec<MarketFeatureDelta>,
    pub(crate) detail_deltas: Vec<MarketFeatureDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ResearchInputManifest {
    pub(crate) schema_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) research_packet_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) run_scope: Option<String>,
    #[serde(default)]
    pub(crate) candidate_bundle_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub(crate) market_feature_delta_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub(crate) market_regime_context_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub(crate) hypothesis_harness_result_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub(crate) historical_replay_run_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub(crate) historical_replay_run_index_refs: Vec<ResearchArtifactRef>,
    #[serde(default)]
    pub(crate) runtime_budget_policy: ResearchRuntimeBudgetPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ResearchArtifactRef {
    pub(crate) uri: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ResearchRuntimeBudgetPolicy {
    pub(crate) max_candidate_bundle_count: usize,
    pub(crate) max_market_artifact_ref_count: usize,
    pub(crate) max_hypothesis_harness_result_ref_count: usize,
    pub(crate) max_historical_replay_run_ref_count: usize,
    pub(crate) max_replay_run_count: usize,
}
