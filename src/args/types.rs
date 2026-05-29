use std::path::PathBuf;

const DEFAULT_AWS_REGION: &str = "ap-northeast-2";

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Args {
    pub(crate) hypothesis_state_file: Option<PathBuf>,
    pub(crate) hypothesis_state_s3: Option<S3InputArgs>,
    pub(crate) market_feature_delta_summary_file: Option<PathBuf>,
    pub(crate) market_feature_delta_summary_s3: Option<S3InputArgs>,
    pub(crate) market_feature_delta_summary_s3_read_limit: usize,
    pub(crate) market_feature_delta_file: Option<PathBuf>,
    pub(crate) market_feature_delta_s3: Option<S3InputArgs>,
    pub(crate) market_feature_delta_s3_read_limit: usize,
    pub(crate) candidate_bundle_s3: Option<S3InputArgs>,
    pub(crate) candidate_bundle_s3_read_limit: usize,
    pub(crate) historical_replay_run_index_s3: Option<S3InputArgs>,
    pub(crate) historical_replay_run_index_s3_read_limit: usize,
    pub(crate) output_dir: Option<PathBuf>,
    pub(crate) output_s3: Option<S3OutputArgs>,
    pub(crate) allow_no_output: bool,
    pub(crate) research_manifest_output_dir: Option<PathBuf>,
    pub(crate) research_manifest_s3: Option<S3OutputArgs>,
    pub(crate) promotion_gate_enabled: bool,
    pub(crate) promotion_gate_include_retest: bool,
    pub(crate) promotion_gate_min_candidate_score: i64,
    pub(crate) promotion_gate_max_candidates: usize,
    pub(crate) promotion_gate_max_market_refs: usize,
    pub(crate) promotion_gate_max_harness_result_refs: usize,
    pub(crate) promotion_gate_max_replay_runs: usize,
    pub(crate) now_ms: Option<i64>,
    pub(crate) promote_score_threshold: i64,
    pub(crate) promote_abs_delta_threshold_pct: f64,
    pub(crate) retest_abs_delta_threshold_pct: f64,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            hypothesis_state_file: None,
            hypothesis_state_s3: None,
            market_feature_delta_summary_file: None,
            market_feature_delta_summary_s3: None,
            market_feature_delta_summary_s3_read_limit: 3,
            market_feature_delta_file: None,
            market_feature_delta_s3: None,
            market_feature_delta_s3_read_limit: 3,
            candidate_bundle_s3: None,
            candidate_bundle_s3_read_limit: 50,
            historical_replay_run_index_s3: None,
            historical_replay_run_index_s3_read_limit: 20,
            output_dir: None,
            output_s3: None,
            allow_no_output: false,
            research_manifest_output_dir: None,
            research_manifest_s3: None,
            promotion_gate_enabled: false,
            promotion_gate_include_retest: false,
            promotion_gate_min_candidate_score: 60,
            promotion_gate_max_candidates: 10,
            promotion_gate_max_market_refs: 100,
            promotion_gate_max_harness_result_refs: 10,
            promotion_gate_max_replay_runs: 500,
            now_ms: None,
            promote_score_threshold: 60,
            promote_abs_delta_threshold_pct: 5.0,
            retest_abs_delta_threshold_pct: 2.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct S3InputArgs {
    pub(crate) bucket: String,
    pub(crate) region: String,
    pub(crate) prefix: String,
    pub(crate) profile: Option<String>,
    pub(crate) max_keys: usize,
}

impl Default for S3InputArgs {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            region: DEFAULT_AWS_REGION.to_owned(),
            prefix: String::new(),
            profile: None,
            max_keys: 1_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct S3OutputArgs {
    pub(crate) bucket: String,
    pub(crate) region: String,
    pub(crate) prefix: String,
    pub(crate) profile: Option<String>,
}

impl Default for S3OutputArgs {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            region: DEFAULT_AWS_REGION.to_owned(),
            prefix: String::new(),
            profile: None,
        }
    }
}
