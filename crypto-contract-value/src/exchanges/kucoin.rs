use crypto_market_type::MarketType;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

use super::utils::http_get;

static LINEAR_CONTRACT_VALUES: Lazy<HashMap<String, f64>> = Lazy::new(|| {
    // offline data, in case the network is down
    let mut m: HashMap<String, f64> = vec![
        ("10000LADYS/USDT", 1000_f64),
        ("10000SATS/USDT", 1000_f64),
        ("10000STARL/USDT", 10_f64),
        ("1000BONK/USDT", 1000_f64),
        ("1000PEPE2/USDT", 10000_f64),
        ("1INCH/USDT", 1_f64),
        ("AAVE/USDT", 0.01_f64),
        ("ACH/USDT", 100_f64),
        ("ADA/USDT", 10_f64),
        ("AGIX/USDT", 1_f64),
        ("AGLD/USDT", 1_f64),
        ("ALGO/USDT", 1_f64),
        ("ALICE/USDT", 0.1_f64),
        ("ALPHA/USDT", 10_f64),
        ("AMB/USDT", 100_f64),
        ("ANC/USDT", 10_f64),
        ("ANKR/USDT", 10_f64),
        ("ANT/USDT", 1_f64),
        ("APE/USDT", 0.1_f64),
        ("API3/USDT", 0.1_f64),
        ("APT/USDT", 0.1_f64),
        ("AR/USDT", 0.1_f64),
        ("ARB/USDT", 1_f64),
        ("ARK/USDT", 10_f64),
        ("ARPA/USDT", 10_f64),
        ("ASTR/USDT", 10_f64),
        ("ATOM/USDT", 0.1_f64),
        ("AUDIO/USDT", 1_f64),
        ("AVAX/USDT", 0.1_f64),
        ("AXS/USDT", 0.1_f64),
        ("BADGER/USDT", 1_f64),
        ("BAKE/USDT", 10_f64),
        ("BAL/USDT", 0.1_f64),
        ("BAND/USDT", 0.1_f64),
        ("BAT/USDT", 1_f64),
        ("BCH/USDT", 0.01_f64),
        ("BEL/USDT", 1_f64),
        ("BIGTIME/USDT", 10_f64),
        ("BLUR/USDT", 1_f64),
        ("BLZ/USDT", 10_f64),
        ("BNB/USDT", 0.01_f64),
        ("BNT/USDT", 1_f64),
        ("BOB/USDT", 100000_f64),
        ("BOND/USDT", 1_f64),
        ("BSV/USDT", 0.01_f64),
        ("BTC/USDC", 0.0001_f64),
        ("BTC/USDT", 0.001_f64),
        ("BTT/USDT", 1000_f64),
        ("C98/USDT", 1_f64),
        ("CAKE/USDT", 1_f64),
        ("CELO/USDT", 1_f64),
        ("CELR/USDT", 10_f64),
        ("CETUS/USDT", 10_f64),
        ("CFX/USDT", 10_f64),
        ("CHR/USDT", 1_f64),
        ("CHZ/USDT", 1_f64),
        ("CKB/USDT", 100_f64),
        ("COCOS/USDT", 1_f64),
        ("COMBO/USDT", 1_f64),
        ("COMP/USDT", 0.01_f64),
        ("CRO/USDT", 1_f64),
        ("CRV/USDT", 1_f64),
        ("CTSI/USDT", 1_f64),
        ("CVC/USDT", 10_f64),
        ("CYBER/USDT", 1_f64),
        ("DAR/USDT", 10_f64),
        ("DASH/USDT", 0.01_f64),
        ("DC/USDT", 1000_f64),
        ("DENT/USDT", 100_f64),
        ("DGB/USDT", 10_f64),
        ("DODO/USDT", 10_f64),
        ("DOGE/USDT", 100_f64),
        ("DOT/USDT", 1_f64),
        ("DUSK/USDT", 10_f64),
        ("DYDX/USDT", 0.1_f64),
        ("EDU/USDT", 1_f64),
        ("EGLD/USDT", 0.01_f64),
        ("ENJ/USDT", 1_f64),
        ("ENS/USDT", 0.1_f64),
        ("EOS/USDT", 1_f64),
        ("ERN/USDT", 1_f64),
        ("ETC/USDT", 0.1_f64),
        ("ETH/USDC", 0.001_f64),
        ("ETH/USDT", 0.01_f64),
        ("FET/USDT", 10_f64),
        ("FIL/USDT", 0.1_f64),
        ("FLM/USDT", 10_f64),
        ("FLOKI/USDT", 100000_f64),
        ("FLOW/USDT", 0.1_f64),
        ("FLUX/USDT", 1_f64),
        ("FRONT/USDT", 10_f64),
        ("FTM/USDT", 1_f64),
        ("FTT/USDT", 1_f64), // changed at 2023-08-13
        ("FXS/USDT", 0.1_f64),
        ("GAL/USDT", 0.1_f64),
        ("GALA/USDT", 1_f64),
        ("GAS/USDT", 1_f64),
        ("GFT/USDT", 100_f64),
        ("GLMR/USDT", 10_f64),
        ("GMT/USDT", 1_f64),
        ("GMX/USDT", 0.01_f64),
        ("GNS/USDT", 1_f64),
        ("GRT/USDT", 1_f64),
        ("GTC/USDT", 1_f64),
        ("HBAR/USDT", 10_f64),
        ("HFT/USDT", 1_f64),
        ("HIFI/USDT", 1_f64),
        ("HIGH/USDT", 1_f64),
        ("HOOK/USDT", 1_f64),
        ("HOT/USDT", 1000_f64),
        ("ICP/USDT", 0.01_f64),
        ("ICX/USDT", 10_f64),
        ("ID/USDT", 10_f64),
        ("IDEX/USDT", 10_f64),
        ("IMX/USDT", 1_f64),
        ("INJ/USDT", 1_f64),
        ("IOST/USDT", 100_f64),
        ("IOTX/USDT", 10_f64),
        ("JASMY/USDT", 10_f64),
        ("JOE/USDT", 1_f64),
        ("JST/USDT", 100_f64),
        ("KAS/USDT", 100_f64),
        ("KAVA/USDT", 0.1_f64),
        ("KDA/USDT", 0.1_f64),
        ("KEEP/USDT", 1_f64),
        ("KEY/USDT", 100_f64),
        ("KLAY/USDT", 1_f64),
        ("KNC/USDT", 1_f64),
        ("KSM/USDT", 0.01_f64),
        ("LDO/USDT", 0.1_f64),
        ("LEVER/USDT", 1000_f64),
        ("LINA/USDT", 10_f64),
        ("LINK/USDT", 0.1_f64),
        ("LIT/USDT", 1_f64),
        ("LOOKS/USDT", 1_f64),
        ("LOOM/USDT", 10_f64),
        ("LPT/USDT", 1_f64),
        ("LQTY/USDT", 1_f64),
        ("LRC/USDT", 1_f64),
        ("LTC/USDT", 0.1_f64),
        ("LUNA/USDT", 1_f64),
        ("LUNC/USDT", 1000_f64),
        ("MAGIC/USDT", 1_f64),
        ("MANA/USDT", 1_f64),
        ("MASK/USDT", 0.1_f64),
        ("MATIC/USDT", 10_f64),
        ("MAV/USDT", 1_f64),
        ("MBL/USDT", 100_f64),
        ("MDT/USDT", 10_f64),
        ("MEME/USDT", 100_f64),
        ("MINA/USDT", 1_f64),
        ("MIR/USDT", 0.1_f64),
        ("MKR/USDT", 0.001_f64),
        ("MTL/USDT", 1_f64),
        ("NEAR/USDT", 0.1_f64),
        ("NEO/USDT", 0.1_f64),
        ("NFT/USDT", 100000_f64),
        ("NKN/USDT", 10_f64),
        ("NMR/USDT", 0.1_f64),
        ("NTRN/USDT", 10_f64),
        ("NYM/USDT", 1_f64),
        ("OCEAN/USDT", 1_f64),
        ("OGN/USDT", 10_f64),
        ("OMG/USDT", 0.1_f64),
        ("ONE/USDT", 10_f64),
        ("ONT/USDT", 1_f64),
        ("OP/USDT", 1_f64),
        ("ORBS/USDT", 100_f64),
        ("ORDI/USDT", 0.1_f64),
        ("OXT/USDT", 10_f64),
        ("PAXG/USDT", 0.001_f64),
        ("PENDLE/USDT", 1_f64),
        ("PEOPLE/USDT", 10_f64),
        ("PEPE/USDT", 520000_f64),
        ("PERP/USDT", 1_f64),
        ("PHB/USDT", 1_f64),
        ("POGAI/USDT", 10000_f64),
        ("POLYX/USDT", 10_f64),
        ("POWR/USDT", 10_f64),
        ("PYTH/USDT", 10_f64),
        ("QNT/USDT", 0.01_f64),
        ("QTUM/USDT", 0.1_f64),
        ("RAD/USDT", 1_f64),
        ("RDNT/USDT", 10_f64),
        ("REEF/USDT", 100_f64),
        ("REN/USDT", 10_f64),
        ("RIF/USDT", 10_f64),
        ("RNDR/USDT", 1_f64),
        ("ROSE/USDT", 10_f64),
        ("RSR/USDT", 100_f64),
        ("RUNE/USDT", 0.1_f64),
        ("RVN/USDT", 10_f64),
        ("SAND/USDT", 1_f64),
        ("SEI/USDT", 10_f64),
        ("SFP/USDT", 10_f64),
        ("SHIB/USDT", 100000_f64),
        ("SKL/USDT", 10_f64),
        ("SLP/USDT", 10_f64),
        ("SNT/USDT", 10_f64),
        ("SNX/USDT", 0.1_f64),
        ("SOL/USDT", 0.1_f64),
        ("SOS/USDT", 100000_f64),
        ("SPELL/USDT", 1000_f64),
        ("SRM/USDT", 1_f64),
        ("SSV/USDT", 0.1_f64),
        ("STEEM/USDT", 10_f64),
        ("STG/USDT", 1_f64),
        ("STMX/USDT", 1000_f64),
        ("STORJ/USDT", 1_f64),
        ("STPT/USDT", 10_f64),
        ("STRAX/USDT", 1_f64),
        ("STX/USDT", 1_f64),
        ("SUI/USDT", 1_f64),
        ("SUSHI/USDT", 1_f64),
        ("SXP/USDT", 1_f64),
        ("T/USDT", 100_f64),
        ("THETA/USDT", 0.1_f64),
        ("TIA/USDT", 1_f64),
        ("TLM/USDT", 1_f64),
        ("TOKEN/USDT", 100_f64),
        ("TOMO/USDT", 1_f64),
        ("TON/USDT", 1_f64),
        ("TRB/USDT", 0.1_f64),
        ("TRU/USDT", 10_f64),
        ("TRX/USDT", 100_f64),
        ("TURBO/USDT", 10000_f64),
        ("TWT/USDT", 1_f64),
        ("UMA/USDT", 1_f64),
        ("UNFI/USDT", 0.1_f64),
        ("UNI/USDT", 1_f64),
        ("USDC/USDT", 1_f64),
        ("VET/USDT", 100_f64),
        ("VGX/USDT", 10_f64),
        ("VRA/USDT", 10_f64),
        ("WAVES/USDT", 0.1_f64),
        ("WAXP/USDT", 1_f64),
        ("WLD/USDT", 1_f64),
        ("WOJAK/USDT", 10000_f64),
        ("WOO/USDT", 1_f64),
        ("WSM/USDT", 100_f64),
        ("XCN/USDT", 10_f64),
        ("XEC/USDT", 100000_f64),
        ("XEM/USDT", 1_f64),
        ("XLM/USDT", 10_f64),
        ("XMR/USDT", 0.01_f64),
        ("XRP/USDT", 10_f64),
        ("XTZ/USDT", 1_f64),
        ("XVG/USDT", 100_f64),
        ("XVS/USDT", 1_f64),
        ("YFI/USDT", 0.0001_f64),
        ("YGG/USDT", 0.1_f64),
        ("ZEC/USDT", 0.01_f64),
        ("ZIL/USDT", 10_f64),
        ("ZRX/USDT", 10_f64),
    ]
    .into_iter()
    .map(|x| (x.0.to_string(), x.1))
    .collect();

    let from_online = fetch_linear_multipliers();
    for (pair, contract_value) in from_online {
        m.insert(pair, contract_value);
    }

    m
});

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct SwapMarket {
    symbol: String,
    multiplier: f64,
    isInverse: bool,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct ResponseMsg {
    code: String,
    data: Vec<SwapMarket>,
}

// get the multiplier field from linear markets
fn fetch_linear_multipliers() -> BTreeMap<String, f64> {
    let mut mapping: BTreeMap<String, f64> = BTreeMap::new();

    if let Ok(txt) = http_get("https://api-futures.kucoin.com/api/v1/contracts/active") {
        if let Ok(resp) = serde_json::from_str::<ResponseMsg>(&txt) {
            for swap_market in resp.data.iter().filter(|x| !x.isInverse) {
                mapping.insert(
                    crypto_pair::normalize_pair(&swap_market.symbol, "kucoin").unwrap(),
                    swap_market.multiplier,
                );
            }
        }
    }

    mapping
}

pub(crate) fn get_contract_value(market_type: MarketType, pair: &str) -> Option<f64> {
    match market_type {
        MarketType::InverseSwap | MarketType::InverseFuture => Some(1.0),
        MarketType::LinearSwap => Some(LINEAR_CONTRACT_VALUES[pair]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::fetch_linear_multipliers;

    #[ignore]
    #[test]
    fn linear() {
        let mut mapping = fetch_linear_multipliers();
        for (key, value) in super::LINEAR_CONTRACT_VALUES.iter() {
            if !mapping.contains_key(key) {
                mapping.insert(key.to_string(), *value);
            }
        }
        for (pair, contract_value) in &mapping {
            println!("(\"{pair}\", {contract_value}_f64),");
        }
    }
}
