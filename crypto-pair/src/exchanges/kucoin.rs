use crypto_market_type::MarketType;

pub(crate) fn normalize_currency(currency: &str) -> String {
    if currency == "XBT" {
        "BTC"
    } else if currency == "BCHSV" {
        "BSV"
    } else if currency == "ETH2" {
        "ksETH"
    } else if currency == "R" {
        "REV"
    } else if currency == "WAX" {
        "WAXP"
    } else if currency == "LOKI" {
        "OXEN"
    } else if currency == "GALAX" {
        "GALA"
    } else {
        currency
    }
    .to_uppercase()
}

pub(crate) fn normalize_pair(symbol: &str) -> Option<String> {
    let (base, quote) = if symbol.ends_with("USDM") {
        // inverse swap
        (symbol.strip_suffix("USDM").unwrap().to_string(), "USD".to_string())
    } else if symbol.ends_with("USDTM") || symbol.ends_with("USDCM") {
        // linear swap
        let base = &symbol[0..symbol.len() - 5];
        let quote = &symbol[symbol.len() - 5..symbol.len() - 1];
        (base.to_string(), quote.to_string())
    } else if symbol[(symbol.len() - 2)..].parse::<i64>().is_ok() {
        // inverse future
        let base = &symbol[..symbol.len() - 4];
        (base.to_string(), "USD".to_string())
    } else if symbol.contains('-') {
        // spot
        let v: Vec<&str> = symbol.split('-').collect();
        (v[0].to_string(), v[1].to_string())
    } else {
        panic!("Unknown symbol {symbol}");
    };

    Some(format!("{}/{}", normalize_currency(&base), normalize_currency(&quote)))
}

pub(crate) fn get_market_type(symbol: &str) -> MarketType {
    if symbol.ends_with("USDM") {
        MarketType::InverseSwap
    } else if symbol.ends_with("USDTM") || symbol.ends_with("USDCM") {
        MarketType::LinearSwap
    } else if symbol[(symbol.len() - 2)..].parse::<i64>().is_ok() {
        MarketType::InverseFuture
    } else if symbol.contains('-') {
        MarketType::Spot
    } else {
        MarketType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use crypto_market_type::MarketType;

    #[test]
    fn test_get_market_type() {
        assert_eq!(MarketType::Spot, super::get_market_type("ETH2-ETH"));
        assert_eq!(MarketType::LinearSwap, super::get_market_type("XBTUSDTM"));
        assert_eq!(MarketType::LinearSwap, super::get_market_type("XBTUSDCM"));
    }

    #[test]
    fn test_normalize_pair() {
        assert_eq!("KSETH/ETH", super::normalize_pair("ETH2-ETH").unwrap());
        assert_eq!("BTC/USDT", super::normalize_pair("XBTUSDTM").unwrap());
        assert_eq!("BTC/USDC", super::normalize_pair("XBTUSDCM").unwrap());
    }
}
