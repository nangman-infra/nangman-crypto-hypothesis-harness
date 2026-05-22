use intel_candidate_app::error::{AppError, AppResult};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct S3InputArgs {
    pub(crate) bucket: String,
    pub(crate) region: String,
    pub(crate) prefix: String,
    pub(crate) endpoint: Option<String>,
    pub(crate) force_path_style: bool,
    pub(crate) profile: Option<String>,
    pub(crate) max_keys: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct S3OutputArgs {
    pub(crate) bucket: String,
    pub(crate) region: String,
    pub(crate) prefix: String,
    pub(crate) endpoint: Option<String>,
    pub(crate) force_path_style: bool,
    pub(crate) profile: Option<String>,
}

pub(crate) fn parse_args(mut values: impl Iterator<Item = String>) -> AppResult<Args> {
    let mut args = Args {
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
    };
    let mut state_s3 = s3_input_args();
    let mut delta_summary_s3 = s3_input_args();
    let mut delta_s3 = s3_input_args();
    let mut candidate_bundle_s3 = s3_input_args();
    let mut historical_replay_run_index_s3 = s3_input_args();
    let mut s3 = S3OutputArgs {
        bucket: String::new(),
        region: DEFAULT_AWS_REGION.to_owned(),
        prefix: String::new(),
        endpoint: None,
        force_path_style: false,
        profile: None,
    };
    let mut research_manifest_s3 = S3OutputArgs {
        bucket: String::new(),
        region: DEFAULT_AWS_REGION.to_owned(),
        prefix: String::new(),
        endpoint: None,
        force_path_style: false,
        profile: None,
    };
    while let Some(arg) = values.next() {
        match arg.as_str() {
            "--hypothesis-state-file" => {
                args.hypothesis_state_file = Some(absolute_path_arg(
                    values.next(),
                    "--hypothesis-state-file requires an absolute path",
                )?);
            }
            "--market-feature-delta-summary-file" => {
                args.market_feature_delta_summary_file = Some(absolute_path_arg(
                    values.next(),
                    "--market-feature-delta-summary-file requires an absolute path",
                )?);
            }
            "--market-feature-delta-file" => {
                args.market_feature_delta_file = Some(absolute_path_arg(
                    values.next(),
                    "--market-feature-delta-file requires an absolute path",
                )?);
            }
            "--hypothesis-state-s3-bucket" => {
                state_s3.bucket = next_string(
                    &mut values,
                    "--hypothesis-state-s3-bucket requires a bucket",
                )?;
            }
            "--hypothesis-state-s3-prefix" => {
                state_s3.prefix = next_string(
                    &mut values,
                    "--hypothesis-state-s3-prefix requires a prefix",
                )?;
            }
            "--hypothesis-state-s3-region" => {
                state_s3.region = next_string(
                    &mut values,
                    "--hypothesis-state-s3-region requires a region",
                )?;
            }
            "--hypothesis-state-s3-endpoint" => {
                state_s3.endpoint = Some(next_string(
                    &mut values,
                    "--hypothesis-state-s3-endpoint requires a URL",
                )?);
            }
            "--hypothesis-state-s3-force-path-style" => state_s3.force_path_style = true,
            "--hypothesis-state-s3-max-keys" => {
                state_s3.max_keys =
                    positive_usize(values.next(), "--hypothesis-state-s3-max-keys")?;
            }
            "--market-feature-delta-summary-s3-bucket" => {
                delta_summary_s3.bucket = next_string(
                    &mut values,
                    "--market-feature-delta-summary-s3-bucket requires a bucket",
                )?;
            }
            "--market-feature-delta-summary-s3-prefix" => {
                delta_summary_s3.prefix = next_string(
                    &mut values,
                    "--market-feature-delta-summary-s3-prefix requires a prefix",
                )?;
            }
            "--market-feature-delta-summary-s3-region" => {
                delta_summary_s3.region = next_string(
                    &mut values,
                    "--market-feature-delta-summary-s3-region requires a region",
                )?;
            }
            "--market-feature-delta-summary-s3-endpoint" => {
                delta_summary_s3.endpoint = Some(next_string(
                    &mut values,
                    "--market-feature-delta-summary-s3-endpoint requires a URL",
                )?);
            }
            "--market-feature-delta-summary-s3-force-path-style" => {
                delta_summary_s3.force_path_style = true
            }
            "--market-feature-delta-summary-s3-max-keys" => {
                delta_summary_s3.max_keys =
                    positive_usize(values.next(), "--market-feature-delta-summary-s3-max-keys")?;
            }
            "--market-feature-delta-summary-s3-read-limit" => {
                args.market_feature_delta_summary_s3_read_limit = positive_usize(
                    values.next(),
                    "--market-feature-delta-summary-s3-read-limit",
                )?;
            }
            "--market-feature-delta-s3-bucket" => {
                delta_s3.bucket = next_string(
                    &mut values,
                    "--market-feature-delta-s3-bucket requires a bucket",
                )?;
            }
            "--market-feature-delta-s3-prefix" => {
                delta_s3.prefix = next_string(
                    &mut values,
                    "--market-feature-delta-s3-prefix requires a prefix",
                )?;
            }
            "--market-feature-delta-s3-region" => {
                delta_s3.region = next_string(
                    &mut values,
                    "--market-feature-delta-s3-region requires a region",
                )?;
            }
            "--market-feature-delta-s3-endpoint" => {
                delta_s3.endpoint = Some(next_string(
                    &mut values,
                    "--market-feature-delta-s3-endpoint requires a URL",
                )?);
            }
            "--market-feature-delta-s3-force-path-style" => delta_s3.force_path_style = true,
            "--market-feature-delta-s3-max-keys" => {
                delta_s3.max_keys =
                    positive_usize(values.next(), "--market-feature-delta-s3-max-keys")?;
            }
            "--market-feature-delta-s3-read-limit" => {
                args.market_feature_delta_s3_read_limit =
                    positive_usize(values.next(), "--market-feature-delta-s3-read-limit")?;
            }
            "--candidate-bundle-s3-bucket" => {
                candidate_bundle_s3.bucket = next_string(
                    &mut values,
                    "--candidate-bundle-s3-bucket requires a bucket",
                )?;
            }
            "--candidate-bundle-s3-prefix" => {
                candidate_bundle_s3.prefix = next_string(
                    &mut values,
                    "--candidate-bundle-s3-prefix requires a prefix",
                )?;
            }
            "--candidate-bundle-s3-region" => {
                candidate_bundle_s3.region = next_string(
                    &mut values,
                    "--candidate-bundle-s3-region requires a region",
                )?;
            }
            "--candidate-bundle-s3-endpoint" => {
                candidate_bundle_s3.endpoint = Some(next_string(
                    &mut values,
                    "--candidate-bundle-s3-endpoint requires a URL",
                )?);
            }
            "--candidate-bundle-s3-force-path-style" => candidate_bundle_s3.force_path_style = true,
            "--candidate-bundle-s3-max-keys" => {
                candidate_bundle_s3.max_keys =
                    positive_usize(values.next(), "--candidate-bundle-s3-max-keys")?;
            }
            "--candidate-bundle-s3-read-limit" => {
                args.candidate_bundle_s3_read_limit =
                    positive_usize(values.next(), "--candidate-bundle-s3-read-limit")?;
            }
            "--historical-replay-run-index-s3-bucket" => {
                historical_replay_run_index_s3.bucket = next_string(
                    &mut values,
                    "--historical-replay-run-index-s3-bucket requires a bucket",
                )?;
            }
            "--historical-replay-run-index-s3-prefix" => {
                historical_replay_run_index_s3.prefix = next_string(
                    &mut values,
                    "--historical-replay-run-index-s3-prefix requires a prefix",
                )?;
            }
            "--historical-replay-run-index-s3-region" => {
                historical_replay_run_index_s3.region = next_string(
                    &mut values,
                    "--historical-replay-run-index-s3-region requires a region",
                )?;
            }
            "--historical-replay-run-index-s3-endpoint" => {
                historical_replay_run_index_s3.endpoint = Some(next_string(
                    &mut values,
                    "--historical-replay-run-index-s3-endpoint requires a URL",
                )?);
            }
            "--historical-replay-run-index-s3-force-path-style" => {
                historical_replay_run_index_s3.force_path_style = true
            }
            "--historical-replay-run-index-s3-max-keys" => {
                historical_replay_run_index_s3.max_keys =
                    positive_usize(values.next(), "--historical-replay-run-index-s3-max-keys")?;
            }
            "--historical-replay-run-index-s3-read-limit" => {
                args.historical_replay_run_index_s3_read_limit =
                    positive_usize(values.next(), "--historical-replay-run-index-s3-read-limit")?;
            }
            "--output-dir" => {
                args.output_dir = Some(absolute_path_arg(
                    values.next(),
                    "--output-dir requires an absolute path",
                )?);
            }
            "--output-s3-bucket" => {
                s3.bucket = next_string(&mut values, "--output-s3-bucket requires a bucket")?
            }
            "--output-s3-region" => {
                s3.region = next_string(&mut values, "--output-s3-region requires a region")?
            }
            "--output-s3-prefix" => {
                s3.prefix = next_string(&mut values, "--output-s3-prefix requires a prefix")?
            }
            "--output-s3-endpoint" => {
                s3.endpoint = Some(next_string(
                    &mut values,
                    "--output-s3-endpoint requires a URL",
                )?)
            }
            "--output-s3-force-path-style" => s3.force_path_style = true,
            "--research-manifest-s3-bucket" => {
                research_manifest_s3.bucket = next_string(
                    &mut values,
                    "--research-manifest-s3-bucket requires a bucket",
                )?
            }
            "--research-manifest-s3-region" => {
                research_manifest_s3.region = next_string(
                    &mut values,
                    "--research-manifest-s3-region requires a region",
                )?
            }
            "--research-manifest-s3-prefix" => {
                research_manifest_s3.prefix = next_string(
                    &mut values,
                    "--research-manifest-s3-prefix requires a prefix",
                )?
            }
            "--research-manifest-s3-endpoint" => {
                research_manifest_s3.endpoint = Some(next_string(
                    &mut values,
                    "--research-manifest-s3-endpoint requires a URL",
                )?)
            }
            "--research-manifest-s3-force-path-style" => {
                research_manifest_s3.force_path_style = true
            }
            "--promotion-gate-enabled" => args.promotion_gate_enabled = true,
            "--promotion-gate-include-retest" => args.promotion_gate_include_retest = true,
            "--promotion-gate-min-candidate-score" => {
                args.promotion_gate_min_candidate_score =
                    non_negative_i64(values.next(), "--promotion-gate-min-candidate-score")?;
            }
            "--promotion-gate-max-candidates" => {
                args.promotion_gate_max_candidates =
                    positive_usize(values.next(), "--promotion-gate-max-candidates")?;
            }
            "--promotion-gate-max-market-refs" => {
                args.promotion_gate_max_market_refs =
                    positive_usize(values.next(), "--promotion-gate-max-market-refs")?;
            }
            "--promotion-gate-max-harness-result-refs" => {
                args.promotion_gate_max_harness_result_refs =
                    positive_usize(values.next(), "--promotion-gate-max-harness-result-refs")?;
            }
            "--promotion-gate-max-replay-runs" => {
                args.promotion_gate_max_replay_runs =
                    positive_usize(values.next(), "--promotion-gate-max-replay-runs")?;
            }
            "--aws-profile" => {
                let profile = next_string(&mut values, "--aws-profile requires a profile")?;
                state_s3.profile = Some(profile.clone());
                delta_summary_s3.profile = Some(profile.clone());
                delta_s3.profile = Some(profile.clone());
                candidate_bundle_s3.profile = Some(profile.clone());
                historical_replay_run_index_s3.profile = Some(profile.clone());
                s3.profile = Some(profile.clone());
                research_manifest_s3.profile = Some(profile);
            }
            "--now-ms" => args.now_ms = Some(non_negative_i64(values.next(), "--now-ms")?),
            "--promote-score-threshold" => {
                args.promote_score_threshold =
                    non_negative_i64(values.next(), "--promote-score-threshold")?;
            }
            "--promote-abs-delta-threshold-pct" => {
                args.promote_abs_delta_threshold_pct =
                    non_negative_f64(values.next(), "--promote-abs-delta-threshold-pct")?;
            }
            "--retest-abs-delta-threshold-pct" => {
                args.retest_abs_delta_threshold_pct =
                    non_negative_f64(values.next(), "--retest-abs-delta-threshold-pct")?;
            }
            "-h" | "--help" => return Err(AppError::config(help_text())),
            other => {
                return Err(AppError::config(format!(
                    "unknown argument: {other}\n\n{}",
                    help_text()
                )));
            }
        }
    }
    if !state_s3.bucket.trim().is_empty() {
        args.hypothesis_state_s3 = Some(state_s3);
    }
    if !delta_summary_s3.bucket.trim().is_empty() {
        args.market_feature_delta_summary_s3 = Some(delta_summary_s3);
    }
    if !delta_s3.bucket.trim().is_empty() {
        args.market_feature_delta_s3 = Some(delta_s3);
    }
    if !candidate_bundle_s3.bucket.trim().is_empty() {
        args.candidate_bundle_s3 = Some(candidate_bundle_s3);
    }
    if !historical_replay_run_index_s3.bucket.trim().is_empty() {
        args.historical_replay_run_index_s3 = Some(historical_replay_run_index_s3);
    }
    if args.hypothesis_state_file.is_none() && args.hypothesis_state_s3.is_none() {
        return Err(AppError::config(
            "--hypothesis-state-file or --hypothesis-state-s3-bucket/--hypothesis-state-s3-prefix is required",
        ));
    }
    if args.output_dir.is_none() && s3.bucket.trim().is_empty() {
        return Err(AppError::config(
            "at least one output target is required: --output-dir or --output-s3-bucket",
        ));
    }
    if !s3.bucket.trim().is_empty() {
        args.output_s3 = Some(s3);
    }
    if !research_manifest_s3.bucket.trim().is_empty() {
        args.research_manifest_s3 = Some(research_manifest_s3);
    }
    if args.promotion_gate_enabled {
        if args.candidate_bundle_s3.is_none() {
            return Err(AppError::config(
                "--candidate-bundle-s3-bucket/--candidate-bundle-s3-prefix is required when --promotion-gate-enabled is set",
            ));
        }
        if args.research_manifest_s3.is_none() {
            return Err(AppError::config(
                "--research-manifest-s3-bucket/--research-manifest-s3-prefix is required when --promotion-gate-enabled is set",
            ));
        }
    }
    Ok(args)
}

fn s3_input_args() -> S3InputArgs {
    S3InputArgs {
        bucket: String::new(),
        region: DEFAULT_AWS_REGION.to_owned(),
        prefix: String::new(),
        endpoint: None,
        force_path_style: false,
        profile: None,
        max_keys: 1_000,
    }
}

fn help_text() -> &'static str {
    r#"hypothesis-harness-app
Usage:
  hypothesis-harness-app \
    --hypothesis-state-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
    --hypothesis-state-s3-prefix hypothesis-state/schema=intel_candidate_hypothesis_state_v1/ \
    --market-feature-delta-summary-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
    --market-feature-delta-summary-s3-prefix market_feature_delta_summary/ \
    --market-feature-delta-summary-s3-read-limit 3 \
    --market-feature-delta-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
    --market-feature-delta-s3-prefix market_feature_delta/ \
    --market-feature-delta-s3-read-limit 3 \
    --output-s3-bucket nangman-crypto-dev-research-<account-suffix> \
    --output-s3-prefix hypothesis-harness/ \
    --promotion-gate-enabled \
    --promotion-gate-include-retest \
    --candidate-bundle-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
    --candidate-bundle-s3-prefix candidate-evidence-bundle/priority=p2/ \
    --historical-replay-run-index-s3-bucket nangman-crypto-dev-research-<account-suffix> \
    --historical-replay-run-index-s3-prefix replay-run-index/ \
    --research-manifest-s3-bucket nangman-crypto-dev-research-<account-suffix> \
    --research-manifest-s3-prefix research-input-manifest/

Runs the cheapest deterministic harness over hypothesis_state records and writes
hypothesis-harness-result plus hypothesis-harness-report. The app prefers
market_feature_delta_summary for the hot path, falls back to detail deltas when
summary has no fresh match, never reuses stale pre-hypothesis artifacts, never
creates orders, and never treats hypothesis_state as a new raw event. When the
promotion gate is enabled, RETEST/PROMOTE harness pressure can materialize a
research_input_manifest_v1 for bounded p2 research replay."#
}

fn absolute_path_arg(value: Option<String>, message: &str) -> AppResult<PathBuf> {
    let value = value.ok_or_else(|| AppError::config(message))?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(AppError::config(format!(
            "{message}; got {}",
            path.display()
        )));
    }
    Ok(path)
}

fn next_string(
    values: &mut impl Iterator<Item = String>,
    message: &'static str,
) -> AppResult<String> {
    let value = values.next().ok_or_else(|| AppError::config(message))?;
    if value.trim().is_empty() {
        return Err(AppError::config(message));
    }
    Ok(value)
}

fn non_negative_i64(value: Option<String>, name: &str) -> AppResult<i64> {
    let raw = value.ok_or_else(|| AppError::config(format!("{name} requires a number")))?;
    let parsed = raw
        .parse::<i64>()
        .map_err(|_| AppError::config(format!("{name} must be an integer")))?;
    if parsed < 0 {
        return Err(AppError::config(format!("{name} must be non-negative")));
    }
    Ok(parsed)
}

fn non_negative_f64(value: Option<String>, name: &str) -> AppResult<f64> {
    let raw = value.ok_or_else(|| AppError::config(format!("{name} requires a number")))?;
    let parsed = raw
        .parse::<f64>()
        .map_err(|_| AppError::config(format!("{name} must be a number")))?;
    if parsed < 0.0 {
        return Err(AppError::config(format!("{name} must be non-negative")));
    }
    Ok(parsed)
}

fn positive_usize(value: Option<String>, name: &str) -> AppResult<usize> {
    let raw = value.ok_or_else(|| AppError::config(format!("{name} requires a number")))?;
    let parsed = raw
        .parse::<usize>()
        .map_err(|_| AppError::config(format!("{name} must be a positive integer")))?;
    if parsed == 0 {
        return Err(AppError::config(format!(
            "{name} must be greater than zero"
        )));
    }
    Ok(parsed)
}
