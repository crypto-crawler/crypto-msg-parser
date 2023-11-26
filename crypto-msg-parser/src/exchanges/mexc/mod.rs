mod mexc_spot;
mod mexc_swap;

use std::collections::HashMap;

use crypto_market_type::MarketType;

use crate::{OrderBookMsg, TradeMsg,CandlestickMsg};

use serde_json::Value;
use simple_error::SimpleError;

pub(super) const EXCHANGE_NAME: &str = "mexc";

pub(crate) fn extract_symbol(msg: &str) -> Result<String, SimpleError> {
    if let Ok(arr) = serde_json::from_str::<Vec<Value>>(msg) {
        Ok(arr[1]["symbol"].as_str().unwrap().to_string())
    } else if let Ok(json_obj) = serde_json::from_str::<HashMap<String, Value>>(msg) {
        if json_obj.contains_key("code") && json_obj.contains_key("data") {
            // RESTful
            let code = json_obj["code"].as_i64().unwrap();
            if code != 0 && code != 200 {
                return Err(SimpleError::new(format!("Error HTTP response {msg}")));
            }
            let data = json_obj["data"].as_object().unwrap();
            if let Some(symbol) = data.get("symbol") {
                Ok(symbol.as_str().unwrap().to_string())
            } else {
                Ok("NONE".to_string())
            }
        } else if let Some(symbol) = json_obj.get("symbol") {
            Ok(symbol.as_str().unwrap().to_string())
        } else {
            Err(SimpleError::new(format!("Unknown message format {msg}")))
        }
    } else {
        Err(SimpleError::new(format!("Unknown message format {msg}")))
    }
}

pub(crate) fn extract_timestamp(msg: &str) -> Result<Option<i64>, SimpleError> {
    if let Ok(arr) = serde_json::from_str::<Vec<Value>>(msg) {
        let channel = arr[0].as_str().unwrap();
        match channel {
            "push.symbol" => {
                let data = arr[1]["data"].as_object().unwrap();
                if let Some(deals) = data.get("deals") {
                    let raw_trades = deals.as_array().unwrap();
                    let timestamp =
                        raw_trades.iter().map(|raw_trade| raw_trade["t"].as_i64().unwrap()).max();

                    if timestamp.is_none() {
                        Err(SimpleError::new(format!("deals is empty in {msg}")))
                    } else {
                        Ok(timestamp)
                    }
                } else {
                    Ok(None)
                }
            }
            "push.kline" => {
                let data = arr[1]["data"].as_object().unwrap();
                let t = data["t"].as_i64().unwrap();
                Ok(Some(t * 1000))
            }
            _ => Err(SimpleError::new(format!("Unknown channel {channel} in {msg}"))),
        }
    } else if let Ok(json_obj) = serde_json::from_str::<HashMap<String, Value>>(msg) {
        if json_obj.contains_key("code") && json_obj.contains_key("data") {
            // RESTful
            let code = json_obj["code"].as_i64().unwrap();
            if code != 0 && code != 200 {
                return Err(SimpleError::new(format!("Error HTTP response {msg}")));
            }
            let data = json_obj["data"].as_object().unwrap();
            Ok(data.get("timestamp").map(|x| x.as_i64().unwrap()))
        } else {
            // websocket
            let channel = json_obj["channel"].as_str().unwrap();
            if let Some(x) = json_obj.get("ts") {
                Ok(Some(x.as_i64().unwrap()))
            } else if channel == "push.kline" {
                let data = json_obj["data"].as_object().unwrap();
                if let Some(tdt) = data.get("tdt") {
                    Ok(Some(tdt.as_i64().unwrap()))
                } else {
                    Ok(Some(data["t"].as_i64().unwrap() * 1000))
                }
            } else if let Some(deals) = json_obj["data"].get("deals") {
                let timestamp = deals
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|raw_trade| raw_trade["t"].as_i64().unwrap())
                    .max();
                if timestamp.is_none() {
                    Err(SimpleError::new(format!("deals is empty in {msg}")))
                } else {
                    Ok(timestamp)
                }
            } else {
                Ok(None)
            }
        }
    } else {
        Err(SimpleError::new(format!("Failed to extract symbol from {msg}")))
    }
}

pub(crate) fn parse_trade(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<TradeMsg>, SimpleError> {
    if market_type == MarketType::Spot {
        mexc_spot::parse_trade(msg)
    } else {
        mexc_swap::parse_trade(market_type, msg)
    }
}

pub(crate) fn parse_l2(
    market_type: MarketType,
    msg: &str,
    timestamp: Option<i64>,
) -> Result<Vec<OrderBookMsg>, SimpleError> {
    if market_type == MarketType::Spot {
        mexc_spot::parse_l2(
            msg,
            timestamp.expect("MEXC Spot orderbook messages don't have timestamp"),
        )
    } else {
        mexc_swap::parse_l2(market_type, msg)
    }
}

pub(crate) fn parse_l2_topk(
    market_type: MarketType,
    msg: &str,
    timestamp: Option<i64>,
) -> Result<Vec<OrderBookMsg>, SimpleError> {
    if market_type == MarketType::Spot {
        mexc_spot::parse_l2_topk(
            msg,
            timestamp.expect("MEXC Spot L2TopK messages don't have timestamp"),
        )
    } else {
        mexc_swap::parse_l2(market_type, msg)
    }
}

pub(crate) fn parse_candlestick(
    market_type:MarketType,
    msg:&str,
    timestamp:Option<i64>,
)-> Result<Vec<CandlestickMsg>,SimpleError>
{
    if market_type == MarketType::Spot {
        mexc_spot::parse_candlestick(
            msg,
            timestamp.expect("MEXC Spot candlestick messages don't have timestamp"),
        )
    } else {
        mexc_swap::parse_candlestick(market_type, msg)
    }
}
