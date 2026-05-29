use super::super::types::{S3InputArgs, S3OutputArgs};

#[derive(Default)]
pub(super) struct PendingS3Args {
    pub(super) state: S3InputArgs,
    pub(super) delta_summary: S3InputArgs,
    pub(super) delta: S3InputArgs,
    pub(super) candidate_bundle: S3InputArgs,
    pub(super) historical_replay_run_index: S3InputArgs,
    pub(super) output: S3OutputArgs,
    pub(super) research_manifest: S3OutputArgs,
}

impl PendingS3Args {
    pub(super) fn apply_profile(&mut self, profile: String) {
        self.state.profile = Some(profile.clone());
        self.delta_summary.profile = Some(profile.clone());
        self.delta.profile = Some(profile.clone());
        self.candidate_bundle.profile = Some(profile.clone());
        self.historical_replay_run_index.profile = Some(profile.clone());
        self.output.profile = Some(profile.clone());
        self.research_manifest.profile = Some(profile);
    }
}
