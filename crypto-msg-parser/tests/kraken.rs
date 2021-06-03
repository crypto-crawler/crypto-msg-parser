mod utils;

use crypto_msg_parser::{parse_l2, parse_trade, MarketType, TradeSide};

#[test]
fn trade() {
    let raw_msg =
        r#"[321,[["57126.70000","0.02063928","1616333924.737428","b","m",""]],"trade","XBT/USD"]"#;
    let trade = &parse_trade("kraken", MarketType::Spot, raw_msg).unwrap()[0];

    crate::utils::check_trade_fields("kraken", MarketType::Spot, "BTC/USD".to_string(), trade);

    assert_eq!(trade.quantity_base, 0.02063928);
    assert_eq!(trade.side, TradeSide::Buy);
}

#[test]
fn l2_orderbook_snapshot() {
    let raw_msg = r#"[320,{"as":[["39090.60000","0.00007039","1622714245.847093"],["39094.90000","0.20000000","1622714255.810162"],["39096.20000","0.25584089","1622714249.255261"]],"bs":[["39071.40000","7.93106570","1622714255.963942"],["39071.30000","0.01090000","1622714249.826684"],["39071.20000","0.76000000","1622714253.348549"]]},"book-25","XBT/USD"]"#;
    let orderbook = &parse_l2("kraken", MarketType::Spot, raw_msg).unwrap()[0];

    assert_eq!(orderbook.asks.len(), 3);
    assert_eq!(orderbook.bids.len(), 3);
    assert!(orderbook.snapshot);

    crate::utils::check_orderbook_fields(
        "kraken",
        MarketType::Spot,
        "BTC/USD".to_string(),
        orderbook,
    );

    assert_eq!(orderbook.timestamp, 1622714245847);

    assert_eq!(orderbook.bids[0][0], 39071.4);
    assert_eq!(orderbook.bids[0][1], 7.93106570);
    assert_eq!(orderbook.bids[0][2], 39071.4 * 7.93106570);

    assert_eq!(orderbook.bids[2][0], 39071.2);
    assert_eq!(orderbook.bids[2][1], 0.76);
    assert_eq!(orderbook.bids[2][2], 39071.2 * 0.76);

    assert_eq!(orderbook.asks[0][0], 39090.6);
    assert_eq!(orderbook.asks[0][1], 0.00007039);
    assert_eq!(orderbook.asks[0][2], 39090.6 * 0.00007039);

    assert_eq!(orderbook.asks[2][0], 39096.2);
    assert_eq!(orderbook.asks[2][1], 0.25584089);
    assert_eq!(orderbook.asks[2][2], 39096.2 * 0.25584089);
}

#[test]
fn l2_orderbook_update() {
    let raw_msg = r#"[320,{"b":[["39071.40000","7.26106570","1622714256.068601"]],"c":"2040672112"},"book-25","XBT/USD"]"#;
    let orderbook = &parse_l2("kraken", MarketType::Spot, raw_msg).unwrap()[0];

    assert_eq!(orderbook.asks.len(), 0);
    assert_eq!(orderbook.bids.len(), 1);
    assert!(!orderbook.snapshot);

    crate::utils::check_orderbook_fields(
        "kraken",
        MarketType::Spot,
        "BTC/USD".to_string(),
        orderbook,
    );

    assert_eq!(orderbook.timestamp, 1622714256068);

    assert_eq!(orderbook.bids[0][0], 39071.4);
    assert_eq!(orderbook.bids[0][1], 7.26106570);
    assert_eq!(orderbook.bids[0][2], 39071.4 * 7.26106570);
}
