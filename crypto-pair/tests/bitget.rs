mod utils;

use crypto_market_type::MarketType;
use crypto_pair::{get_market_type, normalize_currency, normalize_pair};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use utils::http_get;

const EXCHANGE_NAME: &str = "bitget";

// See https://bitgetlimited.github.io/apidoc/en/spot/#get-all-instruments
#[derive(Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct SpotMarket {
    symbol: String,     // symbol Id
    symbolName: String, // symbol name
    baseCoin: String,   // Base coin
    quoteCoin: String,  // Denomination coin
    status: String,     // Status
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

// See https://bitgetlimited.github.io/apidoc/en/mix/#get-all-symbols
#[derive(Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct SwapMarket {
    symbol: String,    // symbol Id
    baseCoin: String,  // Base coin
    quoteCoin: String, // Denomination coin
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

// See https://bitgetlimited.github.io/apidoc/en/spot/#get-all-instruments
fn fetch_spot_markets_raw() -> Vec<SpotMarket> {
    #[derive(Serialize, Deserialize)]
    #[allow(non_snake_case)]
    struct Response {
        code: String,
        msg: String,
        data: Vec<SpotMarket>,
        requestTime: Value,
        #[serde(flatten)]
        extra: HashMap<String, Value>,
    }

    let txt = http_get("https://api.bitget.com/api/spot/v1/public/products").unwrap();
    let resp = serde_json::from_str::<Response>(&txt).unwrap();
    if resp.msg != "success" {
        Vec::new()
    } else {
        resp.data
            .into_iter()
            // Ignored ETH_SPBL and BTC_SPBL for now because they're not tradable
            .filter(|x| x.status == "online" && x.symbol.ends_with("USDT_SPBL"))
            .collect::<Vec<SpotMarket>>()
    }
}

// See https://bitgetlimited.github.io/apidoc/en/mix/#get-all-symbols
// product_type: umcbl, LinearSwap; dmcbl, InverseSwap;
fn fetch_swap_markets_raw(product_type: &str) -> Vec<SwapMarket> {
    #[derive(Serialize, Deserialize)]
    #[allow(non_snake_case)]
    struct Response {
        code: String,
        msg: String,
        data: Vec<SwapMarket>,
        requestTime: Value,
        #[serde(flatten)]
        extra: HashMap<String, Value>,
    }

    let txt = http_get(
        format!("https://api.bitget.com/api/mix/v1/market/contracts?productType={product_type}")
            .as_str(),
    )
    .unwrap();
    let resp = serde_json::from_str::<Response>(&txt).unwrap();
    if resp.msg != "success" { Vec::new() } else { resp.data }
}

#[test]
fn verify_spot_symbols() {
    let markets = fetch_spot_markets_raw();
    for market in markets.iter() {
        let pair = normalize_pair(&market.symbol, EXCHANGE_NAME).unwrap();
        let pair_expected = format!(
            "{}/{}",
            normalize_currency(&market.baseCoin, EXCHANGE_NAME),
            normalize_currency(&market.quoteCoin, EXCHANGE_NAME)
        );

        assert_eq!(pair.as_str(), pair_expected);
        assert_eq!(MarketType::Spot, get_market_type(&market.symbol, EXCHANGE_NAME, None));
    }
}

#[test]
fn verify_swap_symbols() {
    let usdt_linear_markets = fetch_swap_markets_raw("umcbl");
    let usdc_linear_markets = fetch_swap_markets_raw("cmcbl");
    let inverse_markets = fetch_swap_markets_raw("dmcbl");
    let markets = usdt_linear_markets
        .into_iter()
        .chain(usdc_linear_markets)
        .chain(inverse_markets)
        .collect::<Vec<SwapMarket>>();
    for market in markets.iter() {
        let pair = normalize_pair(&market.symbol, EXCHANGE_NAME).unwrap();
        let pair_expected = format!(
            "{}/{}",
            normalize_currency(&market.baseCoin, EXCHANGE_NAME),
            normalize_currency(
                if market.symbol.ends_with("PERP_CMCBL") { "USDC" } else { &market.quoteCoin },
                EXCHANGE_NAME
            )
        );
        assert_eq!(pair.as_str(), pair_expected);

        let market_type = get_market_type(&market.symbol, EXCHANGE_NAME, None);
        assert!(
            market_type == MarketType::LinearSwap
                || market_type == MarketType::InverseSwap
                || market_type == MarketType::InverseFuture
        );
    }
}
