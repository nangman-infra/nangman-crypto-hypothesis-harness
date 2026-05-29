use intel_candidate_app::model::MarketFeatureDelta;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MatchStatus {
    NoSymbolMatch,
    NoFreshMatch,
    Matched,
}

#[derive(Debug, Clone)]
pub(crate) struct MatchedMarketArtifacts<'a> {
    pub(crate) deltas: Vec<&'a MarketFeatureDelta>,
    pub(crate) status: MatchStatus,
}
