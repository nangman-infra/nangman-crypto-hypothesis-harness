use intel_candidate_app::model::{
    IntelCandidateHypothesisState, MarketFeatureDelta, SelectedMarketArtifactTrace,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ArtifactBaseline {
    window_end_ms: i64,
    known_as_of_ms: i64,
}

pub(super) fn artifact_baseline(state: &IntelCandidateHypothesisState) -> ArtifactBaseline {
    state
        .selected_market_artifacts
        .iter()
        .filter(|artifact| is_market_feature_artifact(artifact))
        .max_by_key(|artifact| (artifact.window_end_ms, artifact.known_as_of_ms))
        .map(|artifact| ArtifactBaseline {
            window_end_ms: artifact.window_end_ms,
            known_as_of_ms: artifact.known_as_of_ms,
        })
        .unwrap_or(ArtifactBaseline {
            window_end_ms: state.updated_at_ms,
            known_as_of_ms: state.updated_at_ms,
        })
}

pub(super) fn is_newer_than_baseline(
    delta: &MarketFeatureDelta,
    baseline: &ArtifactBaseline,
) -> bool {
    (delta.window_end_ms, delta.known_as_of_ms) > (baseline.window_end_ms, baseline.known_as_of_ms)
}

fn is_market_feature_artifact(artifact: &SelectedMarketArtifactTrace) -> bool {
    matches!(
        artifact.artifact_type.trim(),
        "market_feature_delta" | "market_feature_delta_summary"
    )
}
