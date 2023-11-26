use crypto_market_type::MarketType;
use crypto_msg_type::MessageType;

use super::EXCHANGE_NAME;

use super::super::utils::calc_quantity_and_volume;
use crypto_message::{CandlestickMsg, Order, OrderBookMsg, TradeMsg, TradeSide};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use simple_error::SimpleError;
use std::collections::HashMap;

// https://mxcdevelop.github.io/APIDoc/contract.api.cn.html#4483df6e28
#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct RawTradeMsg {
    p: f64, // price
    v: f64, // quantity
    T: i64, // 1, buy; 2, sell
    t: i64, // timestamp
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

// https://mxcdevelop.github.io/APIDoc/contract.api.cn.html#a1128a972d
#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct RawOrderbookMsg {
    version: Option<u64>,
    asks: Vec<[f64; 3]>,
    bids: Vec<[f64; 3]>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

// https://mexcdevelop.github.io/apidocs/spot_v2_cn/#k
#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
struct RawCandlestickMsg {
    symbol: String,
    interval: String,
    t: i64,
    o: f64,
    c: f64,
    h: f64,
    l: f64,
    a: f64,
    q: f64,
    //ro:f64,
    //rc:f64,
    //rh:f64,
    //rl:f64,
}

#[derive(Serialize, Deserialize)]
struct WebsocketMsg<T: Sized> {
    channel: String,
    symbol: String,
    ts: i64,
    data: T,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

pub(super) fn parse_trade(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<TradeMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<RawTradeMsg>>(msg).map_err(|_e| {
        SimpleError::new(format!("Failed to deserialize {msg} to WebsocketMsg<RawTradeMsg>"))
    })?;
    let symbol = ws_msg.symbol.as_str();
    let pair = crypto_pair::normalize_pair(symbol, super::EXCHANGE_NAME)
        .ok_or_else(|| SimpleError::new(format!("Failed to normalize {symbol} from {msg}")))?;
    let raw_trade = ws_msg.data;

    let (quantity_base, quantity_quote, _) = calc_quantity_and_volume(
        super::EXCHANGE_NAME,
        market_type,
        &pair,
        raw_trade.p,
        raw_trade.v,
    );

    let trade = TradeMsg {
        exchange: super::EXCHANGE_NAME.to_string(),
        market_type,
        symbol: symbol.to_string(),
        pair,
        msg_type: MessageType::Trade,
        timestamp: raw_trade.t,
        price: raw_trade.p,
        quantity_base,
        quantity_quote,
        quantity_contract: Some(raw_trade.v),
        side: if raw_trade.T == 2 { TradeSide::Sell } else { TradeSide::Buy },
        trade_id: raw_trade.t.to_string(),
        json: msg.to_string(),
    };

    Ok(vec![trade])
}

pub(crate) fn parse_l2(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<OrderBookMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<RawOrderbookMsg>>(msg).map_err(|_e| {
        SimpleError::new(format!("Failed to deserialize {msg} to WebsocketMsg<RawOrderbookMsg>"))
    })?;
    let symbol = ws_msg.symbol.as_str();
    let pair = crypto_pair::normalize_pair(symbol, super::EXCHANGE_NAME)
        .ok_or_else(|| SimpleError::new(format!("Failed to normalize {symbol} from {msg}")))?;

    let msg_type = match ws_msg.channel.as_str() {
        "push.depth.full" => MessageType::L2TopK,
        "push.depth" => MessageType::L2Event,
        _ => panic!("Unsupported channel {}", ws_msg.channel),
    };

    let parse_order = |raw_order: &[f64; 3]| -> Order {
        let price = raw_order[0];
        let quantity = raw_order[1];
        let (quantity_base, quantity_quote, quantity_contract) =
            calc_quantity_and_volume(super::EXCHANGE_NAME, market_type, &pair, price, quantity);
        Order { price, quantity_base, quantity_quote, quantity_contract }
    };

    let orderbook = OrderBookMsg {
        exchange: super::EXCHANGE_NAME.to_string(),
        market_type,
        symbol: symbol.to_string(),
        pair: pair.to_string(),
        msg_type,
        timestamp: ws_msg.ts,
        seq_id: ws_msg.data.version,
        prev_seq_id: None,
        asks: ws_msg.data.asks.iter().map(parse_order).collect::<Vec<Order>>(),
        bids: ws_msg.data.bids.iter().map(parse_order).collect::<Vec<Order>>(),
        snapshot: msg_type == MessageType::L2TopK,
        json: msg.to_string(),
    };

    Ok(vec![orderbook])
}

pub(crate) fn parse_candlestick(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<CandlestickMsg>, SimpleError> {
    let ws_msg = serde_json::from_str::<WebsocketMsg<RawCandlestickMsg>>(msg).map_err(|_e| {
        SimpleError::new(format!("Failed to deserialize {msg} to WebsocketMsg<RawCandlestickMsg>"))
    })?;
    let symbol = ws_msg.symbol.as_str();
    let pair = crypto_pair::normalize_pair(symbol, super::EXCHANGE_NAME)
        .ok_or_else(|| SimpleError::new(format!("Failed to normalize {symbol} from {msg}")))?;
    let mut interval_in_seconds = 0;
    if ws_msg.data.interval.as_str().starts_with("Min") {
        interval_in_seconds =
            ws_msg.data.interval.as_str().strip_prefix("Min").unwrap().parse::<i64>().unwrap() * 60;
    } else if ws_msg.data.interval.as_str().starts_with("Hour") {
        interval_in_seconds =
            ws_msg.data.interval.as_str().strip_prefix("Hour").unwrap().parse::<i64>().unwrap()
                * 60
                * 60;
    } else if ws_msg.data.interval.as_str().starts_with("Day") {
        interval_in_seconds =
            ws_msg.data.interval.as_str().strip_prefix("Day").unwrap().parse::<i64>().unwrap()
                * 60
                * 60
                * 24;
    } else if ws_msg.data.interval.as_str().starts_with("Week") {
        interval_in_seconds =
            ws_msg.data.interval.as_str().strip_prefix("Week").unwrap().parse::<i64>().unwrap()
                * 60
                * 60
                * 24
                * 7;
    } else if ws_msg.data.interval.as_str().starts_with("Month") {
        //how to calculate Month intervals in milliseconds?
        interval_in_seconds =
            ws_msg.data.interval.as_str().strip_prefix("Month").unwrap().parse::<i64>().unwrap()
                * 60
                * 60
                * 24
                * 7
                * 30;
    }
    let (volume, quote_volume) = match market_type {
        MarketType::InverseSwap => {
            let volume = ws_msg.data.q;
            let quote_volume = ws_msg.data.a;
            (volume, quote_volume)
        }
        MarketType::LinearSwap => {
            let contract_value = crypto_contract_value::get_contract_value(
                EXCHANGE_NAME,
                market_type,
                pair.as_str(),
            )
            .unwrap();
            let volume = ws_msg.data.q;
            let quote_volume = ws_msg.data.a;
            (volume * contract_value, quote_volume)
        }
        _ => panic!("Unknown market_type: {}", market_type),
    };

    let candlestick_msg = CandlestickMsg {
        exchange: super::EXCHANGE_NAME.to_string(),
        market_type,
        msg_type: MessageType::Candlestick,
        symbol: symbol.to_string(),
        pair,
        timestamp: ws_msg.data.t * 1000,
        period: ws_msg.data.interval,
        begin_time: ws_msg.data.t * 1000 - interval_in_seconds * 1000,
        open: ws_msg.data.o,
        high: ws_msg.data.h,
        low: ws_msg.data.l,
        close: ws_msg.data.c,
        volume,
        quote_volume: Some(quote_volume),
        json: msg.to_string(),
    };

    Ok(vec![candlestick_msg])
}
