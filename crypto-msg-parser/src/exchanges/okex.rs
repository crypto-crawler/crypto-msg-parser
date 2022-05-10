use crypto_market_type::MarketType;
use crypto_msg_type::MessageType;

use super::utils::calc_quantity_and_volume;
use crate::{FundingRateMsg, Order, OrderBookMsg, TradeMsg, TradeSide};

use chrono::{prelude::*, DateTime};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use simple_error::SimpleError;
use std::collections::HashMap;

const EXCHANGE_NAME: &str = "okex";

// https://www.okex.com/docs/en/#spot_ws-trade
// https://www.okex.com/docs/en/#futures_ws-trade
// https://www.okex.com/docs/en/#ws_swap-trade
// https://www.okex.com/docs/en/#option_ws-trade
#[derive(Serialize, Deserialize)]
struct RawTradeMsg {
    instrument_id: String,
    trade_id: String,
    price: String,
    size: Option<String>,
    qty: Option<String>,
    trade_side: Option<String>, // buy, sell, for option/trades only
    side: Option<String>,       // buy, sell, for other
    timestamp: String,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

// https://www.okex.com/docs/en/#spot_ws-full_depth
// https://www.okex.com/docs/en/#futures_ws-full_depth
// https://www.okex.com/docs/en/#ws_swap-full_depth
// https://www.okex.com/docs/en/#option_ws-full_depth
#[derive(Serialize, Deserialize)]
struct RawOrderbookMsg {
    instrument_id: String,
    timestamp: String,
    asks: Vec<[String; 4]>,
    bids: Vec<[String; 4]>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct RawFundingRateMsg {
    estimated_rate: String,
    funding_rate: String,
    funding_time: String,
    instrument_id: String,
    settlement_time: String,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct WebsocketMsg<T: Sized> {
    table: String,
    data: Vec<T>,
    action: Option<String>, // partial, update
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

pub(crate) fn extract_symbol(_market_type: MarketType, msg: &str) -> Result<String, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<Value>>(msg).map_err(|_e| {
        SimpleError::new(format!(
            "Failed to deserialize {} to WebsocketMsg<Value>",
            msg
        ))
    })?;
    let symbols = ws_msg
        .data
        .iter()
        .map(|v| v["instrument_id"].as_str().unwrap())
        .collect::<Vec<&str>>();
    if symbols.is_empty() {
        Err(SimpleError::new("empty array"))
    } else {
        Ok(symbols[0].to_string())
    }
}

pub(crate) fn extract_timestamp(
    _market_type: MarketType,
    msg: &str,
) -> Result<Option<i64>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<Value>>(msg).map_err(|_e| {
        SimpleError::new(format!(
            "Failed to deserialize {} to WebsocketMsg<Value>",
            msg
        ))
    })?;
    let timestamp = ws_msg.data.iter().fold(std::i64::MIN, |a, x| {
        a.max(
            DateTime::parse_from_rfc3339(x["timestamp"].as_str().unwrap())
                .unwrap()
                .timestamp_millis(),
        )
    });

    if timestamp == std::i64::MIN {
        Err(SimpleError::new(format!("data is empty in {}", msg)))
    } else {
        Ok(Some(timestamp))
    }
}

pub(crate) fn get_msg_type(msg: &str) -> MessageType {
    if let Ok(ws_msg) = serde_json::from_str::<WebsocketMsg<Value>>(msg) {
        let table = ws_msg.table;
        let channel = {
            let arr = table.split('/').collect::<Vec<&str>>();
            arr[1]
        };
        if channel == "trade" {
            MessageType::Trade
        } else if channel == "depth_l2_tbt" {
            MessageType::L2Event
        } else if channel == "depth5" {
            MessageType::L2TopK
        } else if channel == "ticker" {
            MessageType::BBO
        } else if channel == "candle" {
            MessageType::Candlestick
        } else if channel == "funding_rate" {
            MessageType::FundingRate
        } else {
            MessageType::Other
        }
    } else {
        MessageType::Other
    }
}

pub(crate) fn parse_trade(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<TradeMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<RawTradeMsg>>(msg).map_err(|_e| {
        SimpleError::new(format!(
            "Failed to deserialize {} to WebsocketMsg<RawTradeMsg>",
            msg
        ))
    })?;
    let mut trades: Vec<Result<TradeMsg, SimpleError>> = ws_msg
        .data
        .into_iter()
        .map(|raw_trade| {
            let timestamp = DateTime::parse_from_rfc3339(&raw_trade.timestamp).unwrap();
            let price = raw_trade.price.parse::<f64>().unwrap();
            let size = if raw_trade.qty.is_some() {
                raw_trade.qty.clone().unwrap().parse::<f64>().unwrap()
            } else if raw_trade.size.is_some() {
                raw_trade.size.clone().unwrap().parse::<f64>().unwrap()
            } else {
                return Err(SimpleError::new(
                    "qty and size are both missing".to_string(),
                ));
            };
            let side = raw_trade.side.clone().unwrap();
            let pair =
                crypto_pair::normalize_pair(&raw_trade.instrument_id, EXCHANGE_NAME).unwrap();
            let (quantity_base, quantity_quote, _) =
                calc_quantity_and_volume(EXCHANGE_NAME, market_type, &pair, price, size);

            Ok(TradeMsg {
                exchange: EXCHANGE_NAME.to_string(),
                market_type,
                symbol: raw_trade.instrument_id.clone(),
                pair,
                msg_type: MessageType::Trade,
                timestamp: timestamp.timestamp_millis(),
                price,
                quantity_base,
                quantity_quote,
                quantity_contract: if market_type == MarketType::Spot {
                    None
                } else {
                    Some(size)
                },
                side: if side.as_str() == "sell" {
                    TradeSide::Sell
                } else {
                    TradeSide::Buy
                },
                trade_id: raw_trade.trade_id.to_string(),
                json: serde_json::to_string(&raw_trade).unwrap(),
            })
        })
        .collect();

    if trades.len() == 1 {
        if let Ok(v) = trades[0].as_mut() {
            v.json = msg.to_string();
        }
    }
    trades.into_iter().collect()
}

pub(crate) fn parse_funding_rate(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<FundingRateMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<RawFundingRateMsg>>(msg).map_err(|_e| {
        SimpleError::new(format!(
            "Failed to deserialize {} to WebsocketMsg<RawFundingRateMsg>",
            msg
        ))
    })?;

    let mut rates: Vec<FundingRateMsg> = ws_msg
        .data
        .into_iter()
        .map(|raw_msg| {
            let funding_time = DateTime::parse_from_rfc3339(&raw_msg.funding_time).unwrap();
            FundingRateMsg {
                exchange: EXCHANGE_NAME.to_string(),
                market_type,
                symbol: raw_msg.instrument_id.clone(),
                pair: crypto_pair::normalize_pair(&raw_msg.instrument_id, EXCHANGE_NAME).unwrap(),
                msg_type: MessageType::FundingRate,
                timestamp: Utc::now().timestamp_millis(),
                funding_rate: raw_msg.funding_rate.parse::<f64>().unwrap(),
                funding_time: funding_time.timestamp_millis(),
                estimated_rate: Some(raw_msg.estimated_rate.parse::<f64>().unwrap()),
                json: serde_json::to_string(&raw_msg).unwrap(),
            }
        })
        .collect();

    if rates.len() == 1 {
        rates[0].json = msg.to_string();
    }
    Ok(rates)
}

pub(crate) fn parse_l2(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<OrderBookMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<RawOrderbookMsg>>(msg).map_err(|_e| {
        SimpleError::new(format!(
            "Failed to deserialize {} to WebsocketMsg<RawOrderbookMsg>",
            msg
        ))
    })?;
    let snapshot = ws_msg.action.unwrap() == "partial";
    debug_assert_eq!(ws_msg.data.len(), 1);

    let mut orderbooks = ws_msg
        .data
        .iter()
        .map(|raw_orderbook| {
            let symbol = raw_orderbook.instrument_id.clone();
            let pair = crypto_pair::normalize_pair(&symbol, EXCHANGE_NAME).unwrap();
            let timestamp = DateTime::parse_from_rfc3339(&raw_orderbook.timestamp).unwrap();

            let parse_order = |raw_order: &[String; 4]| -> Order {
                let price = raw_order[0].parse::<f64>().unwrap();
                let quantity = raw_order[1].parse::<f64>().unwrap();
                let (quantity_base, quantity_quote, quantity_contract) =
                    calc_quantity_and_volume(EXCHANGE_NAME, market_type, &pair, price, quantity);

                Order {
                    price,
                    quantity_base,
                    quantity_quote,
                    quantity_contract,
                }
            };

            OrderBookMsg {
                exchange: EXCHANGE_NAME.to_string(),
                market_type,
                symbol,
                pair: pair.clone(),
                msg_type: MessageType::L2Event,
                timestamp: timestamp.timestamp_millis(),
                seq_id: None,
                prev_seq_id: None,
                asks: raw_orderbook
                    .asks
                    .iter()
                    .map(|x| parse_order(x))
                    .collect::<Vec<Order>>(),
                bids: raw_orderbook
                    .bids
                    .iter()
                    .map(|x| parse_order(x))
                    .collect::<Vec<Order>>(),
                snapshot,
                json: serde_json::to_string(raw_orderbook).unwrap(),
            }
        })
        .collect::<Vec<OrderBookMsg>>();

    if orderbooks.len() == 1 {
        orderbooks[0].json = msg.to_string();
    }
    Ok(orderbooks)
}
