use crypto_market_type::MarketType;

#[allow(clippy::manual_map)]
pub(crate) fn normalize_pair(symbol: &str) -> Option<String> {
    if symbol.contains('_') {
        Some(symbol.replace('_', "/").to_uppercase())
    } else if let Some(base) = symbol.strip_suffix("usdt") {
        Some(format!("{}/USDT", base.to_uppercase()))
    } else if let Some(base) = symbol.strip_suffix("usdc") {
        Some(format!("{}/USDC", base.to_uppercase()))
    } else if let Some(base) = symbol.strip_suffix("qc") {
        Some(format!("{}/QC", base.to_uppercase()))
    } else if let Some(base) = symbol.strip_suffix("btc") {
        Some(format!("{}/BTC", base.to_uppercase()))
    } else {
        None
    }
}

pub(crate) fn get_market_type(symbol: &str) -> MarketType {
    let lowercase = symbol.to_lowercase();
    if lowercase.as_str() == symbol { MarketType::Spot } else { MarketType::LinearSwap }
}

#[cfg(test)]
mod tests {
    use super::normalize_pair;

    #[test]
    fn test_normalize_pair() {
        assert_eq!(Some("BTC/USDT".to_string()), normalize_pair("btc_usdt"));
        assert_eq!(Some("BTC/USDT".to_string()), normalize_pair("btcusdt"));
        assert_eq!(Some("BTC/USDT".to_string()), normalize_pair("BTC_USDT"));
    }
}
