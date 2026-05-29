use intel_candidate_app::error::{AppError, AppResult};

use super::super::types::Args;
use super::pending::PendingS3Args;

pub(super) fn finalize_args(mut args: Args, pending: PendingS3Args) -> AppResult<Args> {
    if !pending.state.bucket.trim().is_empty() {
        args.hypothesis_state_s3 = Some(pending.state);
    }
    if !pending.delta_summary.bucket.trim().is_empty() {
        args.market_feature_delta_summary_s3 = Some(pending.delta_summary);
    }
    if !pending.delta.bucket.trim().is_empty() {
        args.market_feature_delta_s3 = Some(pending.delta);
    }
    if !pending.candidate_bundle.bucket.trim().is_empty() {
        args.candidate_bundle_s3 = Some(pending.candidate_bundle);
    }
    if !pending.historical_replay_run_index.bucket.trim().is_empty() {
        args.historical_replay_run_index_s3 = Some(pending.historical_replay_run_index);
    }
    if args.hypothesis_state_file.is_none() && args.hypothesis_state_s3.is_none() {
        return Err(AppError::config(
            "--hypothesis-state-file or --hypothesis-state-s3-bucket/--hypothesis-state-s3-prefix is required",
        ));
    }
    if args.output_dir.is_none() && pending.output.bucket.trim().is_empty() && !args.allow_no_output
    {
        return Err(AppError::config(
            "at least one output target is required: --output-dir or --output-s3-bucket; use --allow-no-output only for explicit smoke validation",
        ));
    }
    if !pending.output.bucket.trim().is_empty() {
        args.output_s3 = Some(pending.output);
    }
    if !pending.research_manifest.bucket.trim().is_empty() {
        args.research_manifest_s3 = Some(pending.research_manifest);
    }
    if args.promotion_gate_enabled {
        if args.candidate_bundle_s3.is_none() {
            return Err(AppError::config(
                "--candidate-bundle-s3-bucket/--candidate-bundle-s3-prefix is required when --promotion-gate-enabled is set",
            ));
        }
        if args.research_manifest_output_dir.is_none() && args.research_manifest_s3.is_none() {
            return Err(AppError::config(
                "--research-manifest-output-dir or --research-manifest-s3-bucket/--research-manifest-s3-prefix is required when --promotion-gate-enabled is set",
            ));
        }
    }
    Ok(args)
}
