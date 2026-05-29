use intel_candidate_app::error::{AppError, AppResult};

use super::help::help_text;
use super::types::Args;

mod finalize;
mod gate;
mod local;
mod pending;
mod s3;

use finalize::finalize_args;
use pending::PendingS3Args;

pub(crate) fn parse_args(mut values: impl Iterator<Item = String>) -> AppResult<Args> {
    let mut args = Args::default();
    let mut pending = PendingS3Args::default();

    while let Some(arg) = values.next() {
        if local::parse_local_path_arg(arg.as_str(), &mut values, &mut args)? {
            continue;
        }
        if s3::parse_s3_arg(arg.as_str(), &mut values, &mut args, &mut pending)? {
            continue;
        }
        if gate::parse_gate_arg(arg.as_str(), &mut values, &mut args)? {
            continue;
        }

        match arg.as_str() {
            "-h" | "--help" => return Err(AppError::config(help_text())),
            other => {
                return Err(AppError::config(format!(
                    "unknown argument: {other}\n\n{}",
                    help_text()
                )));
            }
        }
    }

    finalize_args(args, pending)
}
