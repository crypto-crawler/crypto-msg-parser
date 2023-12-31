use crypto_market_type::MarketType;
use crypto_message::{CandlestickMsg, Order, OrderBookMsg, TradeMsg, TradeSide};
use crypto_msg_type::MessageType;
use serde_json::Value;
use simple_error::SimpleError;
use std::collections::HashMap;

use crate::exchanges::utils::calc_quantity_and_volume;

use super::EXCHANGE_NAME;
use serde::{Deserialize, Serialize};

pub(super) fn extract_timestamp(msg: &str) -> Result<Option<i64>, SimpleError> {
    let obj = serde_json::from_str::<HashMap<String, Value>>(msg)
        .map_err(|_e| SimpleError::new(format!("Failed to parse the JSON string {msg}")))?;
    if let Some(raw_channel) = obj.get("channel") {
        // websocket
        let raw_channel = raw_channel.as_str().unwrap();
        let channel = raw_channel.split('_').nth(1).unwrap();
        match channel {
            "ticker" => Ok(Some(obj["date"].as_str().unwrap().parse::<i64>().unwrap())),
            "depth" => Ok(obj.get("timestamp").map(|x| x.as_i64().unwrap() * 1000)),
            "trades" => {
                let timestamp = obj["data"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x["date"].as_i64().unwrap())
                    .max();
                if let Some(timestamp) = timestamp {
                    Ok(Some(timestamp * 1000))
                } else {
                    Err(SimpleError::new(format!("data is empty in {msg}")))
                }
            }
            "kline" => {
                let timestamp = obj["datas"]["data"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x[0].as_i64().unwrap())
                    .max();
                if timestamp.is_none() {
                    Err(SimpleError::new(format!("data is empty in {msg}")))
                } else {
                    Ok(timestamp)
                }
            }
            _ => Err(SimpleError::new(format!("Failed to extract timestamp from {msg}"))),
        }
    } else {
        // RESTful
        Ok(obj.get("timestamp").map(|x| x.as_i64().unwrap() * 1000))
    }
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct WebsocketMsg<T: Sized> {
    dataType: String,
    channel: String,
    data: Vec<T>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct RawTradeMsg {
    date: i64,
    amount: String,
    price: String,
    trade_type: String, // ask, bid
    #[serde(rename = "type")]
    type_: String, // sell, buy
    tid: i64,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct L2TopKMsg {
    channel: String,
    dataType: String,
    timestamp: i64,
    asks: Vec<[f64; 2]>,
    bids: Vec<[f64; 2]>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct RawCandlestickMsg {
    datas: Data,
    channel: String,
    isSuc: Value,
}

#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct Data {
    data: Vec<[Value; 6]>,
}

pub(super) fn parse_trade(msg: &str) -> Result<Vec<TradeMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<RawTradeMsg>>(msg).map_err(|_e| {
        SimpleError::new(format!("Failed to deserialize {msg} to WebsocketMsg<RawTradeMsg>"))
    })?;
    debug_assert_eq!("trades", ws_msg.dataType);
    debug_assert!(ws_msg.channel.ends_with("_trades"));
    let symbol = ws_msg.channel.split('_').next().unwrap();
    let pair = crypto_pair::normalize_pair(symbol, EXCHANGE_NAME).unwrap();

    let trades: Vec<TradeMsg> = ws_msg
        .data
        .into_iter()
        .map(|raw_trade| {
            let price = raw_trade.price.parse::<f64>().unwrap();
            let quantity_base = raw_trade.amount.parse::<f64>().unwrap();
            TradeMsg {
                exchange: EXCHANGE_NAME.to_string(),
                market_type: MarketType::Spot,
                symbol: symbol.to_string(),
                pair: pair.to_string(),
                msg_type: MessageType::Trade,
                timestamp: raw_trade.date * 1000,
                price,
                quantity_base,
                quantity_quote: price * quantity_base,
                quantity_contract: None,
                side: if raw_trade.type_ == "sell" { TradeSide::Sell } else { TradeSide::Buy },
                trade_id: raw_trade.tid.to_string(),
                json: serde_json::to_string(&raw_trade).unwrap(),
            }
        })
        .collect();

    Ok(trades)
}

pub(super) fn parse_l2(_msg: &str) -> Result<Vec<OrderBookMsg>, SimpleError> {
    Err(SimpleError::new("ZB spor market doesn't provide incrememtal level2 channel"))
}

pub(super) fn parse_l2_topk(msg: &str) -> Result<Vec<OrderBookMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<L2TopKMsg>(msg)
        .map_err(|_e| SimpleError::new(format!("Failed to deserialize {msg} to L2TopKMsg")))?;
    debug_assert_eq!("depth", ws_msg.dataType);
    debug_assert!(ws_msg.channel.ends_with("_depth"));
    let symbol = ws_msg.channel.split('_').next().unwrap();
    let timestamp = ws_msg.timestamp * 1000;
    let pair = crypto_pair::normalize_pair(symbol, EXCHANGE_NAME).unwrap();

    let parse_order = |raw_order: &[f64; 2]| -> Order {
        let price = raw_order[0];
        let quantity_base = raw_order[1];

        Order {
            price,
            quantity_base,
            quantity_quote: price * quantity_base,
            quantity_contract: None,
        }
    };

    let orderbook = OrderBookMsg {
        exchange: EXCHANGE_NAME.to_string(),
        market_type: MarketType::Spot,
        symbol: symbol.to_string(),
        pair,
        msg_type: MessageType::L2TopK,
        timestamp,
        seq_id: None,
        prev_seq_id: None,
        asks: ws_msg.asks.iter().map(parse_order).collect::<Vec<Order>>(),
        bids: ws_msg.bids.iter().map(parse_order).collect::<Vec<Order>>(),
        snapshot: true,
        json: msg.to_string(),
    };
    Ok(vec![orderbook])
}

// * https://www.zb.com/en/api #Market GetKline
pub(super) fn parse_candlestick(msg: &str) -> Result<Vec<CandlestickMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<RawCandlestickMsg>(msg).map_err(SimpleError::from)?;

    let (symbol, period) = {
        let mut arr = ws_msg.channel.split('_');
        (arr.next().unwrap(), arr.last().unwrap())
    };
    let pair = crypto_pair::normalize_pair(symbol, EXCHANGE_NAME).unwrap();

    let mut m_seconds = 0;
    if period.ends_with("min") {
        m_seconds = period.strip_suffix("min").unwrap().parse::<i64>().unwrap() * 60 * 1000;
    } else if period.ends_with("hour") {
        m_seconds = period.strip_suffix("hour").unwrap().parse::<i64>().unwrap() * 60 * 60 * 1000;
    } else if period.ends_with("day") {
        m_seconds =
            period.strip_suffix("day").unwrap().parse::<i64>().unwrap() * 60 * 60 * 24 * 1000;
    } else if period.ends_with("week") {
        m_seconds =
            period.strip_suffix("week").unwrap().parse::<i64>().unwrap() * 60 * 60 * 24 * 7 * 1000;
    }

    let arr = ws_msg.datas.data;
    let mut candlestick_msgs: Vec<CandlestickMsg> = arr
        .into_iter()
        .map(|candlestick_msg| {
            let timestamp = candlestick_msg[0].as_i64().unwrap();
            let begin_time = timestamp - m_seconds;

            let open = candlestick_msg[1].as_f64().unwrap();
            let high = candlestick_msg[2].as_f64().unwrap();
            let low = candlestick_msg[3].as_f64().unwrap();
            let close = candlestick_msg[4].as_f64().unwrap();
            let price = (open + high + low + close) / 4.0;
            let quantity = candlestick_msg[5].as_f64().unwrap();
            let (volume, quote_volume, _none) = calc_quantity_and_volume(
                EXCHANGE_NAME,
                MarketType::Spot,
                pair.as_str(),
                price,
                quantity,
            );

            CandlestickMsg {
                exchange: super::EXCHANGE_NAME.to_string(),
                market_type: MarketType::Spot,
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
