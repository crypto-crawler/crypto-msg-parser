use crypto_market_type::MarketType;
use crypto_msg_type::MessageType;

use crate::{Order, OrderBookMsg};

use super::EXCHANGE_NAME;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use simple_error::SimpleError;
use std::collections::HashMap;

/// price, quantity
pub type RawOrder = [String; 2];

// See https://binance-docs.github.io/apidocs/spot/en/#partial-book-depth-streams
#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct RawL2TopKMsg {
    lastUpdateId: u64, // Last update ID
    asks: Vec<RawOrder>,
    bids: Vec<RawOrder>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct WebsocketMsg<T: Sized> {
    stream: String,
    data: T,
}

// See https://binance-docs.github.io/apidocs/spot/en/#order-book
#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct RawL2SnapshotMsg {
    lastUpdateId: u64, // Last update ID
    bids: Vec<RawOrder>,
    asks: Vec<RawOrder>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

// The @depth20 payload of spot has quite different format from contracts.
pub(super) fn parse_l2_topk(
    msg: &str,
    received_at: Option<i64>,
) -> Result<Vec<OrderBookMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<RawL2TopKMsg>>(msg).map_err(|_e| {
        SimpleError::new(format!("Failed to deserialize {msg} to WebsocketMsg<RawL2TopKMsg>"))
    })?;
    debug_assert!(!ws_msg.stream.starts_with('!'));
    let symbol = ws_msg.stream.as_str().split('@').next().unwrap().to_uppercase();
    let pair = crypto_pair::normalize_pair(&symbol, EXCHANGE_NAME)
        .ok_or_else(|| SimpleError::new(format!("Failed to normalize {} from {}", &symbol, msg)))?;
    let timestamp = received_at.expect("Binance spot L2TopK doesn't have timestamp");

    let parse_order = |raw_order: &RawOrder| -> Order {
        let price = raw_order[0].parse::<f64>().unwrap();
        let quantity_base = raw_order[1].parse::<f64>().unwrap();
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
        symbol,
        pair,
        msg_type: MessageType::L2TopK,
        timestamp,
        seq_id: Some(ws_msg.data.lastUpdateId),
        prev_seq_id: None,
        asks: ws_msg.data.asks.iter().map(parse_order).collect::<Vec<Order>>(),
        bids: ws_msg.data.bids.iter().map(parse_order).collect::<Vec<Order>>(),
        snapshot: true,
        json: msg.to_string(),
    };
    Ok(vec![orderbook])
}

// binance l2 snapshot data is quite large
pub(super) fn parse_l2_snapshot(
    msg: &str,
    symbol: Option<&str>,
    received_at: Option<i64>,
) -> Result<Vec<OrderBookMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<RawL2SnapshotMsg>(msg).map_err(|_e| {
        SimpleError::new(format!("Failed to deserialize {msg} to RawL2SnapshotMsg"))
    })?;

    let pair = crypto_pair::normalize_pair(symbol.unwrap(), EXCHANGE_NAME).ok_or_else(|| {
        SimpleError::new(format!("Failed to normalize {} from {}", symbol.unwrap(), msg))
    })?;
    let timestamp = received_at.expect("Binance spot L2 Snapshot doesn't have timestamp");

    let parse_order = |raw_order: &RawOrder| -> Order {
        let price = raw_order[0].parse::<f64>().unwrap();
        let quantity_base = raw_order[1].parse::<f64>().unwrap();
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
        symbol: symbol.unwrap().to_string(),
        pair,
        msg_type: MessageType::L2Snapshot,
        timestamp,
        seq_id: Some(ws_msg.lastUpdateId),
        prev_seq_id: None,
        asks: ws_msg.asks.iter().map(parse_order).collect::<Vec<Order>>(),
        bids: ws_msg.bids.iter().map(parse_order).collect::<Vec<Order>>(),
        snapshot: true,
        json: msg.to_string(),
    };
    Ok(vec![orderbook])
}
