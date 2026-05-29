use super::logging::log_event;
use crate::args::Args;
use crate::model::ResearchInputManifest;
use crate::output::{write_research_manifest_to_dir, write_research_manifest_to_s3};
use intel_candidate_app::error::AppResult;

pub(crate) async fn handle_research_manifest_outputs(
    args: &Args,
    created_at_ms: i64,
    manifest: &ResearchInputManifest,
    output_files: &mut Vec<String>,
    output_s3_uris: &mut Vec<String>,
) -> AppResult<usize> {
    let (output_path, output_uri) = write_research_manifest_outputs(
        args,
        created_at_ms,
        manifest,
        output_files,
        output_s3_uris,
    )
    .await?;
    if output_path.is_none() && output_uri.is_none() {
        return Ok(0);
    }
    log_event(
        "research_manifest_created",
        serde_json::json!({
            "research_packet_id": manifest.research_packet_id,
            "candidate_bundle_ref_count": manifest.candidate_bundle_refs.len(),
            "market_feature_delta_ref_count": manifest.market_feature_delta_refs.len(),
            "market_regime_context_ref_count": manifest.market_regime_context_refs.len(),
            "hypothesis_harness_result_ref_count": manifest.hypothesis_harness_result_refs.len(),
            "historical_replay_run_index_ref_count": manifest.historical_replay_run_index_refs.len(),
            "output_path": output_path,
            "output_uri": output_uri
        }),
    );
    Ok(1)
}

async fn write_research_manifest_outputs(
    args: &Args,
    created_at_ms: i64,
    manifest: &ResearchInputManifest,
    output_files: &mut Vec<String>,
    output_s3_uris: &mut Vec<String>,
) -> AppResult<(Option<String>, Option<String>)> {
    let mut output_path = None;
    let mut output_uri = None;
    if let Some(output_dir) = args.research_manifest_output_dir.as_deref() {
        let path = write_research_manifest_to_dir(output_dir, created_at_ms, manifest)?;
        output_files.push(path.clone());
        output_path = Some(path);
    }
    if let Some(s3) = args.research_manifest_s3.as_ref() {
        let uri = write_research_manifest_to_s3(s3, created_at_ms, manifest).await?;
        output_s3_uris.push(uri.clone());
        output_uri = Some(uri);
    }
    Ok((output_path, output_uri))
}
