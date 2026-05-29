use crate::args::Args;
use intel_candidate_app::model::IntelCandidateEvidenceBundle;
use std::collections::BTreeSet;

pub(crate) fn eligible_candidate_lifecycle_keys(
    args: &Args,
    bundles: &[IntelCandidateEvidenceBundle],
) -> BTreeSet<String> {
    selected_candidate_bundles(args, bundles)
        .into_iter()
        .map(|bundle| bundle.candidate_lifecycle_key.clone())
        .collect()
}

pub(super) fn selected_candidate_bundles<'a>(
    args: &Args,
    bundles: &'a [IntelCandidateEvidenceBundle],
) -> Vec<&'a IntelCandidateEvidenceBundle> {
    let mut selected = bundles
        .iter()
        .filter(|bundle| bundle.research_eligible)
        .filter(|bundle| bundle.candidate_score >= args.promotion_gate_min_candidate_score)
        .filter(|bundle| !bundle.candidate_lifecycle_key.trim().is_empty())
        .filter(|bundle| !bundle.bundle_key.trim().is_empty())
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        right
            .candidate_score
            .cmp(&left.candidate_score)
            .then_with(|| right.created_at_ms.cmp(&left.created_at_ms))
            .then_with(|| {
                left.candidate_lifecycle_key
                    .cmp(&right.candidate_lifecycle_key)
            })
            .then_with(|| left.bundle_key.cmp(&right.bundle_key))
    });
    selected.dedup_by(|left, right| left.candidate_lifecycle_key == right.candidate_lifecycle_key);
    selected.truncate(args.promotion_gate_max_candidates);
    selected
}
