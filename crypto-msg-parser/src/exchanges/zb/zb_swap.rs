use crypto_market_type::MarketType;
use crypto_message::{CandlestickMsg, Order, OrderBookMsg, TradeMsg, TradeSide};
use crypto_msg_type::MessageType;

use super::EXCHANGE_NAME;
use crate::exchanges::utils::calc_quantity_and_volume;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use simple_error::SimpleError;
use std::collections::HashMap;

pub(super) fn extract_timestamp(
    _market_type: MarketType,
    msg: &str,
) -> Result<Option<i64>, SimpleError> {
    let obj = serde_json::from_str::<HashMap<String, Value>>(msg)
        .map_err(|_e| SimpleError::new(format!("Failed to parse the JSON string {msg}")))?;
    if let Some(raw_channel) = obj.get("channel") {
        // websocket
        let raw_channel = raw_channel.as_str().unwrap();
        let channel = raw_channel.split('.').nth(1).unwrap();
        match channel {
            "Trade" => {
                let timestamp =
                    obj["data"].as_array().unwrap().iter().map(|x| x[3].as_i64().unwrap()).max();
                if let Some(timestamp) = timestamp {
                    Ok(Some(timestamp * 1000))
                } else {
                    Err(SimpleError::new(format!("data is empty in {msg}")))
                }
            }
            "Depth" | "DepthWhole" => {
                Ok(obj["data"].get("time").map(|x| x.as_str().unwrap().parse::<i64>().unwrap()))
            }
            "Ticker" => {
                if raw_channel == "All.Ticker" {
                    let m = obj["data"].as_object().unwrap();
                    let timestamp = m.values().map(|x| x[6].as_i64().unwrap()).max();
                    if let Some(timestamp) = timestamp {
                        Ok(Some(timestamp * 1000))
                    } else {
                        Err(SimpleError::new(format!("data is empty in {msg}")))
                    }
                } else {
                    let timestamp = obj["data"].as_array().unwrap()[6].as_i64().unwrap();
                    Ok(Some(timestamp * 1000))
                }
            }
            _ => {
                if channel.starts_with("KLine_") {
                    let timestamp = obj["data"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_array().unwrap()[5].as_i64().unwrap())
                        .max();
                    if let Some(timestamp) = timestamp {
                        Ok(Some(timestamp * 1000))
                    } else {
                        Err(SimpleError::new(format!("data is empty in {msg}")))
                    }
                } else {
                    Err(SimpleError::new(format!("Failed to extract timestamp from {msg}")))
                }
            }
        }
    } else if obj.contains_key("code") && obj.contains_key("desc") && obj.contains_key("data") {
        // ZB linear_swap RESTful
        Ok(obj["data"].get("time").map(|x| x.as_i64().unwrap()))
    } else {
        Err(SimpleError::new(format!("Unknown message format {msg}")))
    }
}

// doc: https://github.com/ZBFuture/docs/blob/main/API%20V2%20_en.md#86-trade
pub(super) fn parse_trade(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<TradeMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<Vec<[f64; 4]>>>(msg).map_err(|_e| {
        SimpleError::new(format!("Failed to deserialize {msg} to WebsocketMsg<Vec<[f64; 4]>>"))
    })?;
    debug_assert!(ws_msg.channel.ends_with(".Trade"));
    let symbol = ws_msg.channel.split('.').next().unwrap();
    let pair = crypto_pair::normalize_pair(symbol, EXCHANGE_NAME).unwrap();

    let trades: Vec<TradeMsg> = ws_msg
        .data
        .into_iter()
        .map(|raw_trade| {
            let price = raw_trade[0];
            let quantity = raw_trade[1];
            let timestamp = (raw_trade[3] as i64) * 1000;

            let (quantity_base, quantity_quote, quantity_contract) =
                calc_quantity_and_volume(EXCHANGE_NAME, market_type, &pair, price, quantity);

            TradeMsg {
                exchange: EXCHANGE_NAME.to_string(),
                market_type,
                symbol: symbol.to_string(),
                pair: pair.to_string(),
                msg_type: MessageType::Trade,
                timestamp,
                price,
                quantity_base,
                quantity_quote,
                quantity_contract,
                side: if raw_trade[3] < 0.0 { TradeSide::Sell } else { TradeSide::Buy },
                trade_id: timestamp.to_string(),
                json: serde_json::to_string(&raw_trade).unwrap(),
            }
        })
        .collect();

    Ok(trades)
}

#[derive(Serialize, Deserialize)]
struct WebsocketMsg<T: Sized> {
    channel: String,
    data: T,
    #[serde(rename = "type")]
    type_: Option<String>, // Whole
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct Level2Msg {
    time: String,
    #[serde(default)]
    asks: Vec<[f64; 2]>,
    #[serde(default)]
    bids: Vec<[f64; 2]>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct RawCandlestickMsg {
    channel: String,
    data: Vec<[Value; 6]>,
    #[serde(rename = "type")]
    type_: Option<String>, // Whole
}

/// Docs:
/// * https://github.com/ZBFuture/docs/blob/main/API%20V2%20_en.md#84-increment-depth
/// * https://github.com/ZBFuture/docs/blob/main/API%20V2%20_en.md#84-increment-depth
pub(super) fn parse_l2(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<OrderBookMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<Level2Msg>>(msg).map_err(|_e| {
        SimpleError::new(format!("Failed to deserialize {msg} to WebsocketMsg<Level2Msg>"))
    })?;
    let msg_type = if ws_msg.channel.ends_with(".DepthWhole") {
        MessageType::L2TopK
    } else if ws_msg.channel.ends_with(".Depth") {
        MessageType::L2Event
    } else {
        panic!("Unsupported channel: {}", ws_msg.channel);
    };
    let snapshot = if msg_type == MessageType::L2TopK {
        true
    } else if let Some(x) = ws_msg.type_ {
        x == "Whole"
    } else {
        false
    };

    let symbol = ws_msg.channel.split('.').next().unwrap();
    let timestamp = ws_msg.data.time.parse::<i64>().unwrap();
    let pair = crypto_pair::normalize_pair(symbol, EXCHANGE_NAME).unwrap();

    let parse_order = |raw_order: &[f64; 2]| -> Order {
        let price = raw_order[0];
        let quantity = raw_order[1];

        let (quantity_base, quantity_quote, quantity_contract) =
            calc_quantity_and_volume(EXCHANGE_NAME, market_type, &pair, price, quantity);
        Order { price, quantity_base, quantity_quote, quantity_contract }
    };

    let orderbook = OrderBookMsg {
        exchange: EXCHANGE_NAME.to_string(),
        market_type,
        symbol: symbol.to_string(),
        pair: pair.clone(),
        msg_type,
        timestamp,
        seq_id: None,
        prev_seq_id: None,
        asks: ws_msg.data.asks.iter().map(parse_order).collect::<Vec<Order>>(),
        bids: ws_msg.data.bids.iter().map(parse_order).collect::<Vec<Order>>(),
        snapshot,
        json: msg.to_string(),
    };
    Ok(vec![orderbook])
}

/// Docs:
/// * https://github.com/ZBFuture/docs/blob/main/API%20V2%20_en.md#85-candlestick
pub(super) fn parse_candlestick(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<CandlestickMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<RawCandlestickMsg>(msg).map_err(|_e| {
        SimpleError::new(format!("Failed to deserialize {msg} to WebsocketMsg<RawCandlestickMsg>"))
    })?;
    //handle channel string to get symbol and period
    let mut s_temp = ws_msg.channel.split('.');
    let symbol = s_temp.next().unwrap();
    let pair = crypto_pair::normalize_pair(symbol, EXCHANGE_NAME).unwrap();
    let period = s_temp.last().unwrap().split('_').last().unwrap();
    //handle period to get time unit
    let mut m_seconds = 0;
    if period.ends_with('M') {
        m_seconds = period.strip_suffix('M').unwrap().parse::<i64>().unwrap() * 60 * 1000;
    } else if period.ends_with('H') {
        m_seconds = period.strip_suffix('H').unwrap().parse::<i64>().unwrap() * 60 * 60 * 1000;
    } else if period.ends_with('D') {
        m_seconds = period.strip_suffix('D').unwrap().parse::<i64>().unwrap() * 60 * 60 * 24 * 1000;
    }

    let arr = ws_msg.data;
    let mut candlestick_msgs: Vec<CandlestickMsg> = arr
        .into_iter()
        .map(|candlestick_msg| {
            let timestamp = candlestick_msg[5].as_i64().unwrap() * 1000;
            let begin_time = timestamp - m_seconds;
            let open = candlestick_msg[0].as_f64().unwrap();
            let high = candlestick_msg[1].as_f64().unwrap();
            let low = candlestick_msg[2].as_f64().unwrap();
            let close = candlestick_msg[3].as_f64().unwrap();
            let price = (open + high + low + close) / 4.0;
            let quantity = candlestick_msg[4].as_f64().unwrap();

            let (volume, quote_volume, _none) = calc_quantity_and_volume(
                EXCHANGE_NAME,
                market_type,
                pair.as_str(),
                price,
                quantity,
            );

            CandlestickMsg {
                exchange: super::EXCHANGE_NAME.to_string(),
                market_type,
                symbol: symbol.to_string(),
                pair: pair.clone(),
                msg_type: MessageType::Candlestick,
                timestamp,
                begin_time,
                open,
                high,
                low,
                close,
                volume,
                period: period.to_string(),
                quote_volume: Some(crate::round(quote_volume)),
                json: msg.to_string(),
            }
        })
        .collect();

    if candlestick_msgs.len() == 1 {
        candlestick_msgs[0].json = msg.to_string();
    }
    Ok(candlestick_msgs)
}
