use std::collections::BTreeSet;

pub(super) fn canonical_symbol_candidates(symbol: &str) -> Vec<String> {
    let upper = symbol.trim().to_ascii_uppercase();
    let mut values = BTreeSet::from([upper.clone()]);
    for suffix in ["USDT", "USDC", "USD", "BUSD", "BTC", "ETH"] {
        if upper.len() > suffix.len() && upper.ends_with(suffix) {
            values.insert(upper.trim_end_matches(suffix).to_owned());
        }
    }
    values.into_iter().collect()
}
