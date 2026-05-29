use intel_candidate_app::error::AppResult;

use super::super::types::Args;
use super::super::validation::{next_string, positive_usize};
use super::pending::PendingS3Args;

pub(super) fn parse_s3_arg(
    arg: &str,
    values: &mut impl Iterator<Item = String>,
    args: &mut Args,
    pending: &mut PendingS3Args,
) -> AppResult<bool> {
    match arg {
        "--hypothesis-state-s3-bucket" => {
            pending.state.bucket =
                next_string(values, "--hypothesis-state-s3-bucket requires a bucket")?;
        }
        "--hypothesis-state-s3-prefix" => {
            pending.state.prefix =
                next_string(values, "--hypothesis-state-s3-prefix requires a prefix")?;
        }
        "--hypothesis-state-s3-region" => {
            pending.state.region =
                next_string(values, "--hypothesis-state-s3-region requires a region")?;
        }
        "--hypothesis-state-s3-max-keys" => {
            pending.state.max_keys =
                positive_usize(values.next(), "--hypothesis-state-s3-max-keys")?;
        }
        "--market-feature-delta-summary-s3-bucket" => {
            pending.delta_summary.bucket = next_string(
                values,
                "--market-feature-delta-summary-s3-bucket requires a bucket",
            )?;
        }
        "--market-feature-delta-summary-s3-prefix" => {
            pending.delta_summary.prefix = next_string(
                values,
                "--market-feature-delta-summary-s3-prefix requires a prefix",
            )?;
        }
        "--market-feature-delta-summary-s3-region" => {
            pending.delta_summary.region = next_string(
                values,
                "--market-feature-delta-summary-s3-region requires a region",
            )?;
        }
        "--market-feature-delta-summary-s3-max-keys" => {
            pending.delta_summary.max_keys =
                positive_usize(values.next(), "--market-feature-delta-summary-s3-max-keys")?;
        }
        "--market-feature-delta-summary-s3-read-limit" => {
            args.market_feature_delta_summary_s3_read_limit = positive_usize(
                values.next(),
                "--market-feature-delta-summary-s3-read-limit",
            )?;
        }
        "--market-feature-delta-s3-bucket" => {
            pending.delta.bucket =
                next_string(values, "--market-feature-delta-s3-bucket requires a bucket")?;
        }
        "--market-feature-delta-s3-prefix" => {
            pending.delta.prefix =
                next_string(values, "--market-feature-delta-s3-prefix requires a prefix")?;
        }
        "--market-feature-delta-s3-region" => {
            pending.delta.region =
                next_string(values, "--market-feature-delta-s3-region requires a region")?;
        }
        "--market-feature-delta-s3-max-keys" => {
            pending.delta.max_keys =
                positive_usize(values.next(), "--market-feature-delta-s3-max-keys")?;
        }
        "--market-feature-delta-s3-read-limit" => {
            args.market_feature_delta_s3_read_limit =
                positive_usize(values.next(), "--market-feature-delta-s3-read-limit")?;
        }
        "--candidate-bundle-s3-bucket" => {
            pending.candidate_bundle.bucket =
                next_string(values, "--candidate-bundle-s3-bucket requires a bucket")?;
        }
        "--candidate-bundle-s3-prefix" => {
            pending.candidate_bundle.prefix =
                next_string(values, "--candidate-bundle-s3-prefix requires a prefix")?;
        }
        "--candidate-bundle-s3-region" => {
            pending.candidate_bundle.region =
                next_string(values, "--candidate-bundle-s3-region requires a region")?;
        }
        "--candidate-bundle-s3-max-keys" => {
            pending.candidate_bundle.max_keys =
                positive_usize(values.next(), "--candidate-bundle-s3-max-keys")?;
        }
        "--candidate-bundle-s3-read-limit" => {
            args.candidate_bundle_s3_read_limit =
                positive_usize(values.next(), "--candidate-bundle-s3-read-limit")?;
        }
        "--historical-replay-run-index-s3-bucket" => {
            pending.historical_replay_run_index.bucket = next_string(
                values,
                "--historical-replay-run-index-s3-bucket requires a bucket",
            )?;
        }
        "--historical-replay-run-index-s3-prefix" => {
            pending.historical_replay_run_index.prefix = next_string(
                values,
                "--historical-replay-run-index-s3-prefix requires a prefix",
            )?;
        }
        "--historical-replay-run-index-s3-region" => {
            pending.historical_replay_run_index.region = next_string(
                values,
                "--historical-replay-run-index-s3-region requires a region",
            )?;
        }
        "--historical-replay-run-index-s3-max-keys" => {
            pending.historical_replay_run_index.max_keys =
                positive_usize(values.next(), "--historical-replay-run-index-s3-max-keys")?;
        }
        "--historical-replay-run-index-s3-read-limit" => {
            args.historical_replay_run_index_s3_read_limit =
                positive_usize(values.next(), "--historical-replay-run-index-s3-read-limit")?;
        }
        "--output-s3-bucket" => {
            pending.output.bucket = next_string(values, "--output-s3-bucket requires a bucket")?;
        }
        "--output-s3-region" => {
            pending.output.region = next_string(values, "--output-s3-region requires a region")?;
        }
        "--output-s3-prefix" => {
            pending.output.prefix = next_string(values, "--output-s3-prefix requires a prefix")?;
        }
        "--research-manifest-s3-bucket" => {
            pending.research_manifest.bucket =
                next_string(values, "--research-manifest-s3-bucket requires a bucket")?;
        }
        "--research-manifest-s3-region" => {
            pending.research_manifest.region =
                next_string(values, "--research-manifest-s3-region requires a region")?;
        }
        "--research-manifest-s3-prefix" => {
            pending.research_manifest.prefix =
                next_string(values, "--research-manifest-s3-prefix requires a prefix")?;
        }
        "--aws-profile" => {
            let profile = next_string(values, "--aws-profile requires a profile")?;
            pending.apply_profile(profile);
        }
        _ => return Ok(false),
    }
    Ok(true)
}
