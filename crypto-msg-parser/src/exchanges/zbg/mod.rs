mod zbg_spot;
mod zbg_swap;

use crypto_market_type::MarketType;

use crate::{CandlestickMsg, OrderBookMsg, TradeMsg};

use simple_error::SimpleError;

const EXCHANGE_NAME: &str = "zbg";

pub(crate) fn extract_symbol(market_type: MarketType, msg: &str) -> Result<String, SimpleError> {
    if market_type == MarketType::Spot {
        zbg_spot::extract_symbol(msg)
    } else {
        zbg_swap::extract_symbol(market_type, msg)
    }
}

pub(crate) fn extract_timestamp(
    market_type: MarketType,
    msg: &str,
) -> Result<Option<i64>, SimpleError> {
    if market_type == MarketType::Spot {
        zbg_spot::extract_timestamp(msg)
    } else {
        zbg_swap::extract_timestamp(market_type, msg)
    }
}

pub(crate) fn parse_trade(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<TradeMsg>, SimpleError> {
    if market_type == MarketType::Spot {
        zbg_spot::parse_trade(msg)
    } else {
        zbg_swap::parse_trade(market_type, msg)
    }
}

pub(crate) fn parse_l2(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<OrderBookMsg>, SimpleError> {
    if market_type == MarketType::Spot {
        zbg_spot::parse_l2(msg)
    } else {
        zbg_swap::parse_l2(market_type, msg)
    }
}

pub(crate) fn parse_candlestick(
    market_type: MarketType,
    msg: &str,
) -> Result<Vec<CandlestickMsg>, SimpleError> {
    if market_type == MarketType::Spot {
        zbg_spot::parse_candlestick(msg)
    } else {
        zbg_swap::parse_candlestick(market_type, msg)
    }
}
