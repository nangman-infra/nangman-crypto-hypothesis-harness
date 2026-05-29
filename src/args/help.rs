pub(super) fn help_text() -> &'static str {
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
    [--allow-no-output] \
    --promotion-gate-enabled \
    --promotion-gate-include-retest \
    --candidate-bundle-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
    --candidate-bundle-s3-prefix candidate-evidence-bundle/priority=p2/ \
    --historical-replay-run-index-s3-bucket nangman-crypto-dev-research-<account-suffix> \
    --historical-replay-run-index-s3-prefix replay-run-index/ \
    [--research-manifest-output-dir /tmp/nangman-harness-research-manifest] \
    [--research-manifest-s3-bucket nangman-crypto-dev-research-<account-suffix> \
     --research-manifest-s3-prefix research-input-manifest/]

Runs the cheapest deterministic harness over hypothesis_state records and writes
hypothesis-harness-result plus hypothesis-harness-report. The app prefers
market_feature_delta_summary for the hot path, falls back to detail deltas when
summary has no fresh match, never reuses stale pre-hypothesis artifacts, never
creates orders, and never treats hypothesis_state as a new raw event. When the
promotion gate is enabled, RETEST/PROMOTE harness pressure can materialize a
research_input_manifest_v1 for bounded p2 research replay."#
}
