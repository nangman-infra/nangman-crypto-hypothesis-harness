# Hypothesis Harness App

`hypothesis-harness-app` runs the cheapest deterministic validation loop over
stored `intel_candidate_hypothesis_state_v1` records. It combines those states
with current `market_feature_delta_v1` artifacts and writes a verdict plus a run
report.

It does not execute trades, create raw intel events, weaken candidate penalties,
or call LLM research.

```text
hypothesis_state prefix
  + market_feature_delta prefix
  -> deterministic harness
  -> hypothesis_harness_result_v1
  -> hypothesis_harness_report_v1
```

## Local Run

```bash
cd /Volumes/WD/Developments/nangman-crypto/apps/hypothesis-harness-app
cargo run -- \
  --hypothesis-state-file /Volumes/WD/Developments/nangman-crypto/apps/hypothesis-harness-app/testdata/hypothesis-state.sample.jsonl \
  --market-feature-delta-summary-file /Volumes/WD/Developments/nangman-crypto/apps/hypothesis-harness-app/testdata/market-feature-delta-summary.sample.json \
  --market-feature-delta-file /Volumes/WD/Developments/nangman-crypto/apps/hypothesis-harness-app/testdata/market-feature-delta.sample.jsonl \
  --output-dir /tmp/nangman-harness-smoke
```

Harness selection rules:

```text
prefer market_feature_delta_summary for the hot path
fallback to market_feature_delta detail only when summary has no fresh symbol match
ignore artifacts that are not newer than the hypothesis state's last seen market trace
use only the latest l1_run_id snapshot, then collapse to the latest row per metric
```

## Scheduled ECS Run

Use an EventBridge Scheduler or one-shot ECS RunTask with S3 prefix inputs.

```bash
hypothesis-harness-app \
  --hypothesis-state-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
  --hypothesis-state-s3-prefix hypothesis-state/schema=intel_candidate_hypothesis_state_v1/ \
  --hypothesis-state-s3-region ap-northeast-2 \
  --hypothesis-state-s3-max-keys 1000 \
  --market-feature-delta-summary-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
  --market-feature-delta-summary-s3-prefix market_feature_delta_summary/ \
  --market-feature-delta-summary-s3-region ap-northeast-2 \
  --market-feature-delta-summary-s3-max-keys 1000 \
  --market-feature-delta-summary-s3-read-limit 3 \
  --market-feature-delta-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
  --market-feature-delta-s3-prefix market_feature_delta/ \
  --market-feature-delta-s3-region ap-northeast-2 \
  --market-feature-delta-s3-max-keys 1000 \
  --market-feature-delta-s3-read-limit 3 \
  --output-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --output-s3-prefix hypothesis-harness/ \
  --output-s3-region ap-northeast-2
```

Run cadence recommendation:

```text
normal market retest    every 5-15 minutes
low-cost observe pass   every 30-60 minutes
policy/app rollout      one-shot after deployment
```

Every invocation writes a report, even when every hypothesis is `OBSERVE` or the
input prefix is empty. That gives the pipeline a heartbeat that can be monitored.

The harness is intentionally conservative about stale inputs:

```text
no fresh market delta after hypothesis_state.updated_at_ms or selected_market_artifacts
  -> OBSERVE
fresh symbol match exists but score < promote threshold
  -> RETEST or OBSERVE depending on delta threshold
```

## Local Research Manifest Check

Use `--research-manifest-output-dir` with `--promotion-gate-enabled` to validate
`research_input_manifest_v1` locally before allowing a scheduler or task role to
write it to S3.

```bash
hypothesis-harness-app \
  --hypothesis-state-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
  --hypothesis-state-s3-prefix hypothesis-state/schema=intel_candidate_hypothesis_state_v1/ \
  --market-feature-delta-summary-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
  --market-feature-delta-summary-s3-prefix market_feature_delta_summary/ \
  --market-feature-delta-s3-bucket nangman-crypto-dev-market-ingest-l1-<account-suffix> \
  --market-feature-delta-s3-prefix market_feature_delta/ \
  --candidate-bundle-s3-bucket nangman-crypto-dev-intel-candidate-<account-suffix> \
  --candidate-bundle-s3-prefix candidate-evidence-bundle/priority=p2/ \
  --historical-replay-run-index-s3-bucket nangman-crypto-dev-research-<account-suffix> \
  --historical-replay-run-index-s3-prefix replay-run-index/ \
  --output-dir /tmp/nangman-harness \
  --promotion-gate-enabled \
  --promotion-gate-include-retest \
  --research-manifest-output-dir /tmp/nangman-harness-research-manifest
```

Local manifest files are written under:

```text
/tmp/nangman-harness-research-manifest/research-input-manifest/schema=research_input_manifest_v1/...
```

## S3 Outputs

```text
s3://nangman-crypto-dev-research-<account-suffix>/hypothesis-harness/hypothesis-harness-result/schema=hypothesis_harness_result_v1/...
s3://nangman-crypto-dev-research-<account-suffix>/hypothesis-harness/hypothesis-harness-report/schema=hypothesis_harness_report_v1/...
s3://nangman-crypto-dev-research-<account-suffix>/research-input-manifest/schema=research_input_manifest_v1/...
```

## Safety Boundary

```text
PROMOTE means re-score/research-admission candidate, not trade
RETEST means repeat deterministic harness on the next window
OBSERVE means keep state without research spend
PRUNE means exclude until revision or stronger evidence appears
```
