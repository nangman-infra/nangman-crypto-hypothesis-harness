pub(super) fn is_usable_market_artifact_quality(status: &str) -> bool {
    matches!(
        status.trim().to_ascii_lowercase().as_str(),
        "" | "complete" | "partial" | "available"
    )
}
