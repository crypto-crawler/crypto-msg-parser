use crypto_market_type::MarketType;
use crypto_msg_type::MessageType;

use crate::{Order, OrderBookMsg, TradeMsg, TradeSide};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use simple_error::SimpleError;
use std::collections::HashMap;

const EXCHANGE_NAME: &str = "mxc";

// https://github.com/mxcdevelop/APIDoc/blob/master/websocket/spot/websocket-api.md#成交记录
#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct RawTradeMsg {
    p: String, // price
    q: String, // quantity
    T: i64,    // 1, buy; 2, sell
    t: i64,    // timestamp
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct RawOrder {
    p: String,
    q: String,
    a: String,
}

#[derive(Serialize, Deserialize)]
struct PushSymbolData {
    deals: Option<Vec<RawTradeMsg>>,
    asks: Option<Vec<RawOrder>>,
    bids: Option<Vec<RawOrder>>,
}

#[derive(Serialize, Deserialize)]
struct WebsocketMsg<T: Sized> {
    symbol: String,
    data: T,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

pub(super) fn parse_trade(msg: &str) -> Result<Vec<TradeMsg>, SimpleError> {
    let ws_msg: WebsocketMsg<PushSymbolData> = if let Ok(arr) =
        serde_json::from_str::<Vec<Value>>(msg)
            .map_err(|_e| SimpleError::new(format!("Failed to deserialize {} to Vec<Value>", msg)))
    {
        assert_eq!(arr.len(), 2);
        serde_json::from_value(arr[1].clone()).map_err(|_e| {
            SimpleError::new(format!(
                "Failed to deserialize {} to WebsocketMsg<PushSymbolData>",
                arr[1]
            ))
        })?
    } else if let Ok(ws_msg) = serde_json::from_str::<WebsocketMsg<PushSymbolData>>(msg) {
        ws_msg
    } else {
        return Err(SimpleError::new(format!("Failed to parse {}", msg)));
    };
    if ws_msg.data.deals.is_none() {
        return Ok(Vec::new());
    }

    let raw_trades = ws_msg.data.deals.unwrap();
    let symbol = ws_msg.symbol.as_str();
    let pair = crypto_pair::normalize_pair(symbol, EXCHANGE_NAME)
        .ok_or_else(|| SimpleError::new(format!("Failed to normalize {} from {}", symbol, msg)))?;

    let mut trades: Vec<TradeMsg> = raw_trades
        .into_iter()
        .map(|raw_trade| {
            let price = raw_trade.p.parse::<f64>().unwrap();
            let quantity = raw_trade.q.parse::<f64>().unwrap();

            TradeMsg {
                exchange: EXCHANGE_NAME.to_string(),
                market_type: MarketType::Spot,
                symbol: symbol.to_string(),
                pair: pair.clone(),
                msg_type: MessageType::Trade,
                timestamp: raw_trade.t,
                price,
                quantity_base: quantity,
                quantity_quote: price * quantity,
                quantity_contract: None,
                side: if raw_trade.T == 2 {
                    TradeSide::Sell
                } else {
                    TradeSide::Buy
                },
                trade_id: raw_trade.t.to_string(),
                json: serde_json::to_string(&raw_trade).unwrap(),
            }
        })
        .collect();

    if trades.len() == 1 {
        trades[0].json = msg.to_string();
    }
    Ok(trades)
}

fn parse_order(raw_order: &RawOrder) -> Order {
    let price = raw_order.p.parse::<f64>().unwrap();
    let quantity_base = raw_order.q.parse::<f64>().unwrap();
    let quantity_quote = raw_order.a.parse::<f64>().unwrap();

    Order {
        price,
        quantity_base,
        quantity_quote,
        quantity_contract: None,
    }
}

pub(crate) fn parse_l2(msg: &str, timestamp: i64) -> Result<Vec<OrderBookMsg>, SimpleError> {
    let ws_msg: WebsocketMsg<PushSymbolData> = if let Ok(arr) =
        serde_json::from_str::<Vec<Value>>(msg)
            .map_err(|_e| SimpleError::new(format!("Failed to deserialize {} to Vec<Value>", msg)))
    {
        assert_eq!(arr.len(), 2);
        serde_json::from_value(arr[1].clone()).map_err(|_e| {
            SimpleError::new(format!(
                "Failed to deserialize {} to WebsocketMsg<PushSymbolData>",
                arr[1]
            ))
        })?
    } else if let Ok(ws_msg) = serde_json::from_str::<WebsocketMsg<PushSymbolData>>(msg) {
        ws_msg
    } else {
        return Err(SimpleError::new(format!("Failed to parse {}", msg)));
    };
    if ws_msg.data.asks.is_none() && ws_msg.data.bids.is_none() {
        return Ok(Vec::new());
    }

    let symbol = ws_msg.symbol.as_str();
    let pair = crypto_pair::normalize_pair(symbol, EXCHANGE_NAME)
        .ok_or_else(|| SimpleError::new(format!("Failed to normalize {} from {}", symbol, msg)))?;

    let orderbook = OrderBookMsg {
        exchange: EXCHANGE_NAME.to_string(),
        market_type: MarketType::Spot,
        symbol: symbol.to_string(),
        pair,
        msg_type: MessageType::L2Event,
        timestamp,
        seq_id: None,
        prev_seq_id: None,
        asks: if let Some(asks) = ws_msg.data.asks {
            asks.iter().map(parse_order).collect::<Vec<Order>>()
        } else {
            Vec::new()
        },
        bids: if let Some(bids) = ws_msg.data.bids {
            bids.iter().map(parse_order).collect::<Vec<Order>>()
        } else {
            Vec::new()
        },
        snapshot: false,
        json: msg.to_string(),
    };

    Ok(vec![orderbook])
}
