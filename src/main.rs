mod args;
mod hash;
mod io;
mod matching;
mod model;
mod output;
mod promotion_gate;
mod report;
mod run;
mod summary;
#[cfg(test)]
mod tests;
mod time;
mod verdict;

use crate::args::parse_args;
use crate::run::async_run;
use intel_candidate_app::error::AppError;
use std::env;

#[tokio::main]
async fn main() {
    match parse_args(env::args().skip(1)) {
        Ok(args) => match async_run(args).await {
            Ok(summary) => match serde_json::to_string_pretty(&summary) {
                Ok(output) => println!("{output}"),
                Err(error) => exit_error(error.into()),
            },
            Err(error) => exit_error(error),
        },
        Err(error) => exit_error(error),
    }
}

fn exit_error(error: AppError) {
    eprintln!("{error}");
    std::process::exit(1);
}
