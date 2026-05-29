use intel_candidate_app::error::AppResult;

use super::super::types::Args;
use super::super::validation::absolute_path_arg;

pub(super) fn parse_local_path_arg(
    arg: &str,
    values: &mut impl Iterator<Item = String>,
    args: &mut Args,
) -> AppResult<bool> {
    match arg {
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
        "--output-dir" => {
            args.output_dir = Some(absolute_path_arg(
                values.next(),
                "--output-dir requires an absolute path",
            )?);
        }
        "--research-manifest-output-dir" => {
            args.research_manifest_output_dir = Some(absolute_path_arg(
                values.next(),
                "--research-manifest-output-dir requires an absolute path",
            )?);
        }
        _ => return Ok(false),
    }
    Ok(true)
}
