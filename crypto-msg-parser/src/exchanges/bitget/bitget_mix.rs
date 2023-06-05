use crypto_market_type::MarketType;
use crypto_msg_type::MessageType;

use crypto_message::{CandlestickMsg, Order, OrderBookMsg, TradeMsg, TradeSide};

use super::EXCHANGE_NAME;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use simple_error::SimpleError;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct Arg {
    instType: String,
    channel: String,
    instId: String,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct WebsocketMsg<T: Sized> {
    action: String,
    arg: Arg,
    data: Vec<T>,
}

#[derive(Serialize, Deserialize)]
struct RawOrderBook {
    ts: String,
    asks: Vec<[String; 2]>,
    bids: Vec<[String; 2]>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

pub(super) fn extract_symbol(msg: &str) -> Result<String, SimpleError> {
    let obj = serde_json::from_str::<WebsocketMsg<Value>>(msg)
        .map_err(|_e| SimpleError::new(format!("Failed to parse JSON string {msg}")))?;
    let inst_type = obj.arg.instType.as_str();
    let symbol = obj.arg.instId.as_str();
    match inst_type {
        "sp" => Ok(format!("{symbol}_SPBL")),
        "mc" => {
            if symbol.ends_with("USDT") {
                Ok(format!("{symbol}_UMCBL"))
            } else {
                Ok(format!("{symbol}_DMCBL"))
            }
        }
        _ => Err(SimpleError::new(format!("Unsupported instType {inst_type} in {msg}"))),
    }
}

pub(super) fn extract_timestamp(msg: &str) -> Result<Option<i64>, SimpleError> {
    let obj = serde_json::from_str::<WebsocketMsg<Value>>(msg)
        .map_err(|_e| SimpleError::new(format!("Failed to parse JSON string {msg}")))?;
    let timestamp = obj
        .data
        .iter()
        .map(|x| {
            let v = if x.is_array() {
                x[0].clone()
            } else {
                let obj = x.as_object().unwrap();
                if obj.contains_key("ts") {
                    obj["ts"].clone()
                } else if obj.contains_key("systemTime") {
                    obj["systemTime"].clone()
                } else {
                    panic!("Can not find timestamp related fields in {msg}");
                }
            };
            if v.is_string() {
                v.as_str().unwrap().parse::<i64>().unwrap()
            } else if v.is_i64() {
                v.as_i64().unwrap()
            } else {
                panic!("Unsupported data format {msg}");
            }
        })
        .max();
    Ok(timestamp)
}

/// docs:
/// * https://bitgetlimited.github.io/apidoc/en/spot/#trades-channel
/// * https://bitgetlimited.github.io/apidoc/en/mix/#trades-channel
pub(super) fn parse_trade(msg: &str) -> Result<Vec<TradeMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<[String; 4]>>(msg)
        .map_err(|_e| SimpleError::new(format!("Failed to parse JSON string {msg}")))?;
    debug_assert_eq!("trade", ws_msg.arg.channel.as_str());
    let (market_type, symbol) = match ws_msg.arg.instType.as_str() {
        "sp" => (MarketType::Spot, format!("{}_SPBL", ws_msg.arg.instId)),
        "mc" => {
            let (market_type, suffix) = if ws_msg.arg.instId.ends_with("USDT") {
                (MarketType::LinearSwap, "UMCBL")
            } else if ws_msg.arg.instId.ends_with("USD") {
                (MarketType::InverseSwap, "DMCBL")
            } else {
                panic!("Unknown instId {} in {}", ws_msg.arg.instId, msg);
            };
            let symbol = format!("{}_{}", ws_msg.arg.instId, suffix);
            (market_type, symbol)
        }
        _ => panic!("Unknown instType {} in {}", ws_msg.arg.instType, msg),
    };
    let pair = crypto_pair::normalize_pair(&symbol, EXCHANGE_NAME).unwrap();
    let mut trades: Vec<TradeMsg> = ws_msg
        .data
        .into_iter()
        .map(|raw_trade| {
            let timestamp = raw_trade[0].parse::<i64>().unwrap();
            let price = raw_trade[1].parse::<f64>().unwrap();
            let quantity = raw_trade[2].parse::<f64>().unwrap();
            let side =
                if raw_trade[3].as_str() == "sell" { TradeSide::Sell } else { TradeSide::Buy };
            TradeMsg {
                exchange: EXCHANGE_NAME.to_string(),
                market_type,
                symbol: symbol.clone(),
                pair: pair.clone(),
                msg_type: MessageType::Trade,
                timestamp,
                price,
                quantity_base: quantity,
                quantity_quote: quantity * price,
                quantity_contract: if ws_msg.arg.instType.as_str() == "sp" {
                    None
                } else {
                    Some(quantity)
                },
                side,
                // Use timestamp as ID because bitget doesn't have trade_id
                trade_id: timestamp.to_string(),
                json: serde_json::to_string(&raw_trade).unwrap(),
            }
        })
        .collect();
    if trades.len() == 1 {
        trades[0].json = msg.to_string();
    }
    Ok(trades)
}

/// docs:
/// * https://bitgetlimited.github.io/apidoc/en/spot/#depth-channel
/// * https://bitgetlimited.github.io/apidoc/en/mix/#order-book-channel
pub(super) fn parse_l2(msg: &str) -> Result<Vec<OrderBookMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<RawOrderBook>>(msg)
        .map_err(|_e| SimpleError::new(format!("Failed to parse JSON string {msg}")))?;
    let snapshot = ws_msg.action == "snapshot";
    let (market_type, symbol) = match ws_msg.arg.instType.as_str() {
        "sp" => (MarketType::Spot, format!("{}_SPBL", ws_msg.arg.instId)),
        "mc" => {
            let (market_type, suffix) = if ws_msg.arg.instId.ends_with("USDT") {
                (MarketType::LinearSwap, "UMCBL")
            } else if ws_msg.arg.instId.ends_with("USD") {
                (MarketType::InverseSwap, "DMCBL")
            } else {
                panic!("Unknown instId {} in {}", ws_msg.arg.instId, msg);
            };
            let symbol = format!("{}_{}", ws_msg.arg.instId, suffix);
            (market_type, symbol)
        }
        _ => panic!("Unknown instType {} in {}", ws_msg.arg.instType, msg),
    };
    let pair = crypto_pair::normalize_pair(&symbol, EXCHANGE_NAME).unwrap();

    let parse_order = |raw_order: &[String; 2]| -> Order {
        let price = raw_order[0].parse::<f64>().unwrap();
        let quantity = raw_order[1].parse::<f64>().unwrap();
        Order {
            price,
            quantity_base: quantity,
            quantity_quote: quantity * price,
            quantity_contract: if market_type == MarketType::Spot { None } else { Some(quantity) },
        }
    };

    let orderbooks = ws_msg
        .data
        .iter()
        .map(|raw_orderbook| OrderBookMsg {
            exchange: EXCHANGE_NAME.to_string(),
            market_type,
            symbol: symbol.to_string(),
            pair: pair.clone(),
            msg_type: MessageType::L2Event,
            timestamp: raw_orderbook.ts.parse::<i64>().unwrap(),
            seq_id: None,
            prev_seq_id: None,
            asks: raw_orderbook.asks.iter().map(|x| parse_order(x)).collect(),
            bids: raw_orderbook.bids.iter().map(|x| parse_order(x)).collect(),
            snapshot,
            json: serde_json::to_string(raw_orderbook).unwrap(),
        })
        .collect();

    Ok(orderbooks)
}

/// docs：
/// * https://bitgetlimited.github.io/apidoc/en/spot/#candlesticks-channel */
/// * https://bitgetlimited.github.io/apidoc/en/mix/#candlesticks-channel */
pub(super) fn parse_candlestick(msg: &str) -> Result<Vec<CandlestickMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<[String; 6]>>(msg)
        .map_err(|_e| SimpleError::new(format!("Failed to parse JSON string {msg}")))?;
    let channel = ws_msg.arg.channel;
    let period = channel.as_str().strip_prefix("candle").unwrap();
    let time_unit = period.to_string().pop().unwrap();
    let m_seconds = match time_unit {
        's' => period.strip_suffix('s').unwrap().parse::<i64>().unwrap() * 1000,
        'm' => period.strip_suffix('m').unwrap().parse::<i64>().unwrap() * 1000 * 60,
        'd' => period.strip_suffix('d').unwrap().parse::<i64>().unwrap() * 1000 * 24 * 60 * 60,
        _ => 0,
    };

    debug_assert_eq!("candle", &(channel.as_str())[0..6]);
    let (market_type, symbol) = match ws_msg.arg.instType.as_str() {
        "sp" => (MarketType::Spot, format!("{}_SPBL", ws_msg.arg.instId)),
        "mc" => {
            let (market_type, suffix) = if ws_msg.arg.instId.ends_with("USDT") {
                (MarketType::LinearSwap, "UMCBL")
            } else if ws_msg.arg.instId.ends_with("USD") {
                (MarketType::InverseSwap, "DMCBL")
            } else {
                panic!("Unknown instId {} in {}", ws_msg.arg.instId, msg);
            };
            let symbol = format!("{}_{}", ws_msg.arg.instId, suffix);
            (market_type, symbol)
        }
        _ => panic!("Unknown instType {} in {}", ws_msg.arg.instType, msg),
    };
    let pair = crypto_pair::normalize_pair(&symbol, EXCHANGE_NAME).unwrap();
    let candlestick_msgs: Vec<CandlestickMsg> = ws_msg
        .data
        .into_iter()
        .map(|raw_candlestickmsg| {
            let timestamp = raw_candlestickmsg[0].parse::<i64>().unwrap();
            let open = raw_candlestickmsg[1].parse::<f64>().unwrap();
            let high = raw_candlestickmsg[2].parse::<f64>().unwrap();
            let low = raw_candlestickmsg[3].parse::<f64>().unwrap();
            let close = raw_candlestickmsg[4].parse::<f64>().unwrap();
            let volume = raw_candlestickmsg[5].parse::<f64>().unwrap();
            let quote_volume = None;
            CandlestickMsg {
                exchange: EXCHANGE_NAME.to_string(),
                market_type,
                msg_type: MessageType::Candlestick,
                symbol: symbol.to_string(),
                pair: pair.clone(),
                timestamp,
                period: period.to_string(),
                begin_time: timestamp - m_seconds,
                open,
                high,
                low,
                close,
                volume,
                quote_volume,
                json: serde_json::to_string(&raw_candlestickmsg).unwrap(),
            }
        })
        .collect();
    Ok(candlestick_msgs)
}
