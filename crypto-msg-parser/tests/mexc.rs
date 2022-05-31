mod utils;

const EXCHANGE_NAME: &str = "mexc";

#[cfg(test)]
mod trade {
    use super::EXCHANGE_NAME;
    use crypto_market_type::MarketType;
    use crypto_msg_parser::{extract_symbol, extract_timestamp, parse_trade, round, TradeSide};

    #[test]
    fn spot() {
        let raw_msg = r#"{"symbol":"BTC_USDT","data":{"deals":[{"t":1646996447307,"p":"39008.35","q":"0.003533","T":2}]},"channel":"push.deal"}"#;
        let trades = &parse_trade(EXCHANGE_NAME, MarketType::Spot, raw_msg).unwrap();

        assert_eq!(trades.len(), 1);
        let trade = &trades[0];

        crate::utils::check_trade_fields(
            EXCHANGE_NAME,
            MarketType::Spot,
            "BTC/USDT".to_string(),
            extract_symbol(EXCHANGE_NAME, MarketType::Spot, raw_msg).unwrap(),
            trade,
            raw_msg,
        );
        assert_eq!(
            1646996447307,
            extract_timestamp(EXCHANGE_NAME, MarketType::Spot, raw_msg)
                .unwrap()
                .unwrap()
        );

        assert_eq!(trade.timestamp, 1646996447307);
        assert_eq!(trade.price, 39008.35);
        assert_eq!(trade.quantity_base, 0.003533);
        assert_eq!(trade.side, TradeSide::Sell);
    }

    #[test]
    fn linear_swap() {
        let raw_msg = r#"{"channel":"push.deal","data":{"M":1,"O":3,"T":2,"p":39766.5,"t":1646999591755,"v":32},"symbol":"BTC_USDT","ts":1646999591755}"#;
        let trades = &parse_trade(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg).unwrap();

        assert_eq!(trades.len(), 1);
        let trade = &trades[0];

        crate::utils::check_trade_fields(
            EXCHANGE_NAME,
            MarketType::LinearSwap,
            "BTC/USDT".to_string(),
            extract_symbol(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg).unwrap(),
            trade,
            raw_msg,
        );
        assert_eq!(
            1646999591755,
            extract_timestamp(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg)
                .unwrap()
                .unwrap()
        );

        assert_eq!(trade.timestamp, 1646999591755);
        assert_eq!(trade.price, 39766.5);
        assert_eq!(trade.quantity_contract, Some(32.0));
        assert_eq!(trade.quantity_base, 0.0001 * 32.0);
        assert_eq!(trade.quantity_quote, round(0.0001 * 32.0 * 39766.5));
        assert_eq!(trade.side, TradeSide::Sell);
    }

    #[test]
    fn inverse_swap() {
        let raw_msg = r#"{"channel":"push.deal","data":{"M":1,"O":3,"T":2,"p":39885.5,"t":1647000043904,"v":8},"symbol":"BTC_USD","ts":1647000043904}"#;
        let trades = &parse_trade(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg).unwrap();

        assert_eq!(trades.len(), 1);
        let trade = &trades[0];

        crate::utils::check_trade_fields(
            EXCHANGE_NAME,
            MarketType::InverseSwap,
            "BTC/USD".to_string(),
            extract_symbol(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg).unwrap(),
            trade,
            raw_msg,
        );
        assert_eq!(
            1647000043904,
            extract_timestamp(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg)
                .unwrap()
                .unwrap()
        );

        assert_eq!(trade.timestamp, 1647000043904);
        assert_eq!(trade.price, 39885.5);
        assert_eq!(trade.quantity_contract, Some(8.0));
        assert_eq!(trade.quantity_quote, 100.0 * 8.0);
        assert_eq!(trade.quantity_base, 100.0 * 8.0 / 39885.5);
        assert_eq!(trade.side, TradeSide::Sell);
    }
}

#[cfg(test)]
mod l2_event {
    use super::EXCHANGE_NAME;
    use chrono::prelude::*;
    use crypto_market_type::MarketType;
    use crypto_msg_parser::{extract_symbol, extract_timestamp, parse_l2};
    use crypto_msg_type::MessageType;

    #[test]
    fn spot() {
        let raw_msg = r#"{"symbol":"BTC_USDT","data":{"version":"672257402","bids":[{"p":"39763.35","q":"0.054069","a":"2149.96457"}]},"channel":"push.depth"}"#;
        let received_at = Utc::now().timestamp_millis();
        let orderbook =
            &parse_l2(EXCHANGE_NAME, MarketType::Spot, raw_msg, Some(received_at)).unwrap()[0];

        assert_eq!(orderbook.asks.len(), 0);
        assert_eq!(orderbook.bids.len(), 1);
        assert!(!orderbook.snapshot);
        assert_eq!(orderbook.seq_id, Some(672257402));

        crate::utils::check_orderbook_fields(
            EXCHANGE_NAME,
            MarketType::Spot,
            MessageType::L2Event,
            "BTC/USDT".to_string(),
            extract_symbol(EXCHANGE_NAME, MarketType::Spot, raw_msg).unwrap(),
            orderbook,
            raw_msg,
        );
        assert_eq!(
            None,
            extract_timestamp(EXCHANGE_NAME, MarketType::Spot, raw_msg,).unwrap()
        );

        assert_eq!(orderbook.bids[0].price, 39763.35);
        assert_eq!(orderbook.bids[0].quantity_base, 0.054069);
    }

    #[test]
    fn linear_swap_update() {
        let raw_msg = r#"{"channel":"push.depth","data":{"asks":[[39961,0,0],[39961.5,0,0]],"bids":[[39962.5,58272,1]],"version":4702740808},"symbol":"BTC_USDT","ts":1647000258746}"#;
        let orderbook = &parse_l2(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg, None).unwrap()[0];

        assert_eq!(orderbook.asks.len(), 2);
        assert_eq!(orderbook.bids.len(), 1);
        assert!(!orderbook.snapshot);
        assert_eq!(orderbook.timestamp, 1647000258746);
        assert_eq!(orderbook.seq_id, Some(4702740808));

        crate::utils::check_orderbook_fields(
            EXCHANGE_NAME,
            MarketType::LinearSwap,
            MessageType::L2Event,
            "BTC/USDT".to_string(),
            extract_symbol(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg).unwrap(),
            orderbook,
            raw_msg,
        );
        assert_eq!(
            1647000258746,
            extract_timestamp(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg)
                .unwrap()
                .unwrap()
        );

        assert_eq!(orderbook.asks[0].price, 39961.0);
        assert_eq!(orderbook.asks[0].quantity_contract, Some(0.0));
        assert_eq!(orderbook.asks[0].quantity_base, 0.0);
        assert_eq!(orderbook.asks[0].quantity_quote, 0.0);

        assert_eq!(orderbook.bids[0].price, 39962.5);
        assert_eq!(orderbook.bids[0].quantity_contract, Some(58272.0));
        assert_eq!(orderbook.bids[0].quantity_base, 0.0001 * 58272.0);
        assert_eq!(orderbook.bids[0].quantity_quote, 0.0001 * 58272.0 * 39962.5);
    }

    #[test]
    fn inverse_swap_update() {
        let raw_msg = r#"{"channel":"push.depth","data":{"asks":[],"bids":[[39944,943,1]],"version":2768205529},"symbol":"BTC_USD","ts":1647000870946}"#;
        let orderbook =
            &parse_l2(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg, None).unwrap()[0];

        assert_eq!(orderbook.asks.len(), 0);
        assert_eq!(orderbook.bids.len(), 1);
        assert!(!orderbook.snapshot);
        assert_eq!(orderbook.timestamp, 1647000870946);
        assert_eq!(orderbook.seq_id, Some(2768205529));

        crate::utils::check_orderbook_fields(
            EXCHANGE_NAME,
            MarketType::InverseSwap,
            MessageType::L2Event,
            "BTC/USD".to_string(),
            extract_symbol(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg).unwrap(),
            orderbook,
            raw_msg,
        );
        assert_eq!(
            1647000870946,
            extract_timestamp(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg)
                .unwrap()
                .unwrap()
        );

        assert_eq!(orderbook.bids[0].price, 39944.0);
        assert_eq!(orderbook.bids[0].quantity_contract, Some(943.0));
        assert_eq!(orderbook.bids[0].quantity_quote, 100.0 * 943.0);
        assert_eq!(orderbook.bids[0].quantity_base, 100.0 * 943.0 / 39944.0);
    }
}

#[cfg(test)]
mod l2_topk {
    use super::EXCHANGE_NAME;
    use chrono::prelude::*;
    use crypto_market_type::MarketType;
    use crypto_msg_parser::{extract_symbol, extract_timestamp, parse_l2_topk, round};
    use crypto_msg_type::MessageType;

    #[test]
    fn spot() {
        let raw_msg = r#"{"channel":"push.limit.depth","symbol":"BTC_USDT","data":{"asks":[["31623.85","4.846968"],["31624.35","0.646284"],["31624.52","1.63524"],["31625.21","0.8769"],["31625.22","0.3"]],"bids":[["31623.82","0.179844"],["31623.8","0.001104"],["31623.79","0.052344"],["31623.75","1.397784"],["31623.69","0.002592"]]},"depth":5,"version":"1502380137"}"#;
        let received_at = Utc::now().timestamp_millis();
        let orderbook =
            &parse_l2_topk(EXCHANGE_NAME, MarketType::Spot, raw_msg, Some(received_at)).unwrap()[0];

        assert_eq!(orderbook.asks.len(), 5);
        assert_eq!(orderbook.bids.len(), 5);
        assert!(orderbook.snapshot);

        crate::utils::check_orderbook_fields(
            EXCHANGE_NAME,
            MarketType::Spot,
            MessageType::L2TopK,
            "BTC/USDT".to_string(),
            extract_symbol(EXCHANGE_NAME, MarketType::Spot, raw_msg).unwrap(),
            orderbook,
            raw_msg,
        );
        assert_eq!(
            None,
            extract_timestamp(EXCHANGE_NAME, MarketType::Spot, raw_msg,).unwrap()
        );

        assert_eq!(received_at, orderbook.timestamp);
        assert_eq!(orderbook.seq_id, Some(1502380137));
        assert_eq!(orderbook.prev_seq_id, None);

        assert_eq!(orderbook.bids[0].price, 31623.82);
        assert_eq!(orderbook.bids[0].quantity_base, 0.179844);
        assert_eq!(orderbook.bids[0].quantity_quote, 31623.82 * 0.179844);
        assert_eq!(orderbook.bids[0].quantity_contract, None);

        assert_eq!(orderbook.bids[4].price, 31623.69);
        assert_eq!(orderbook.bids[4].quantity_base, 0.002592);
        assert_eq!(orderbook.bids[4].quantity_quote, 31623.69 * 0.002592);
        assert_eq!(orderbook.bids[4].quantity_contract, None);

        assert_eq!(orderbook.asks[0].price, 31623.85);
        assert_eq!(orderbook.asks[0].quantity_base, 4.846968);
        assert_eq!(orderbook.asks[0].quantity_quote, 31623.85 * 4.846968);
        assert_eq!(orderbook.asks[0].quantity_contract, None);

        assert_eq!(orderbook.asks[4].price, 31625.22);
        assert_eq!(orderbook.asks[4].quantity_base, 0.3);
        assert_eq!(orderbook.asks[4].quantity_quote, 31625.22 * 0.3);
        assert_eq!(orderbook.asks[4].quantity_contract, None);
    }

    #[test]
    fn inverse_swap() {
        let raw_msg = r#"{"channel":"push.depth.full","data":{"asks":[[31647.5,1029,1],[31648,378,1],[31648.5,357,1]],"bids":[[31647,154,1],[31646.5,1470,1],[31646,140,1]],"version":3087532764},"symbol":"BTC_USD","ts":1653994948112}"#;
        let orderbook =
            &parse_l2_topk(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg, None).unwrap()[0];

        assert_eq!(orderbook.asks.len(), 3);
        assert_eq!(orderbook.bids.len(), 3);
        assert!(orderbook.snapshot);

        crate::utils::check_orderbook_fields(
            EXCHANGE_NAME,
            MarketType::InverseSwap,
            MessageType::L2TopK,
            "BTC/USD".to_string(),
            extract_symbol(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg).unwrap(),
            orderbook,
            raw_msg,
        );
        assert_eq!(
            1653994948112,
            extract_timestamp(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg)
                .unwrap()
                .unwrap()
        );

        assert_eq!(orderbook.timestamp, 1653994948112);
        assert_eq!(orderbook.seq_id, Some(3087532764));
        assert_eq!(orderbook.prev_seq_id, None);

        assert_eq!(orderbook.bids[0].price, 31647.0);
        assert_eq!(orderbook.bids[0].quantity_base, 100.0 * 154.0 / 31647.0);
        assert_eq!(orderbook.bids[0].quantity_quote, 100.0 * 154.0);
        assert_eq!(orderbook.bids[0].quantity_contract, Some(154.0));

        assert_eq!(orderbook.bids[2].price, 31646.0);
        assert_eq!(orderbook.bids[2].quantity_base, 100.0 * 140.0 / 31646.0);
        assert_eq!(orderbook.bids[2].quantity_quote, 100.0 * 140.0);
        assert_eq!(orderbook.bids[2].quantity_contract, Some(140.0));

        assert_eq!(orderbook.asks[0].price, 31647.5);
        assert_eq!(orderbook.asks[0].quantity_base, 100.0 * 1029.0 / 31647.5);
        assert_eq!(orderbook.asks[0].quantity_quote, 100.0 * 1029.0);
        assert_eq!(orderbook.asks[0].quantity_contract, Some(1029.0));

        assert_eq!(orderbook.asks[2].price, 31648.5);
        assert_eq!(orderbook.asks[2].quantity_base, 100.0 * 357.0 / 31648.5);
        assert_eq!(orderbook.asks[2].quantity_quote, 100.0 * 357.0);
        assert_eq!(orderbook.asks[2].quantity_contract, Some(357.0));
    }

    #[test]
    fn linear_swap() {
        let raw_msg = r#"{"channel":"push.depth.full","data":{"asks":[[31708.5,74950,2],[31709,8817,2],[31709.5,22296,2]],"bids":[[31708,3210,1],[31706,184756,2],[31705.5,10412,1]],"version":5194972869},"symbol":"BTC_USDT","ts":1653995843510}"#;
        let orderbook =
            &parse_l2_topk(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg, None).unwrap()[0];

        assert_eq!(orderbook.asks.len(), 3);
        assert_eq!(orderbook.bids.len(), 3);
        assert!(orderbook.snapshot);

        crate::utils::check_orderbook_fields(
            EXCHANGE_NAME,
            MarketType::LinearSwap,
            MessageType::L2TopK,
            "BTC/USDT".to_string(),
            extract_symbol(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg).unwrap(),
            orderbook,
            raw_msg,
        );
        assert_eq!(
            1653995843510,
            extract_timestamp(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg)
                .unwrap()
                .unwrap()
        );

        assert_eq!(orderbook.timestamp, 1653995843510);
        assert_eq!(orderbook.seq_id, Some(5194972869));
        assert_eq!(orderbook.prev_seq_id, None);

        assert_eq!(orderbook.bids[0].price, 31708.0);
        assert_eq!(orderbook.bids[0].quantity_base, 0.0001 * 3210.0);
        assert_eq!(orderbook.bids[0].quantity_quote, 0.0001 * 3210.0 * 31708.0);
        assert_eq!(orderbook.bids[0].quantity_contract, Some(3210.0));

        assert_eq!(orderbook.bids[2].price, 31705.5);
        assert_eq!(orderbook.bids[2].quantity_base, round(0.0001 * 10412.0));
        assert_eq!(orderbook.bids[2].quantity_quote, 0.0001 * 10412.0 * 31705.5);
        assert_eq!(orderbook.bids[2].quantity_contract, Some(10412.0));

        assert_eq!(orderbook.asks[0].price, 31708.5);
        assert_eq!(orderbook.asks[0].quantity_base, 0.0001 * 74950.0);
        assert_eq!(orderbook.asks[0].quantity_quote, 0.0001 * 74950.0 * 31708.5);
        assert_eq!(orderbook.asks[0].quantity_contract, Some(74950.0));

        assert_eq!(orderbook.asks[2].price, 31709.5);
        assert_eq!(orderbook.asks[2].quantity_base, 0.0001 * 22296.0);
        assert_eq!(orderbook.asks[2].quantity_quote, 0.0001 * 22296.0 * 31709.5);
        assert_eq!(orderbook.asks[2].quantity_contract, Some(22296.0));
    }
}

#[cfg(test)]
mod before_20220311 {
    #[cfg(test)]
    mod trade {
        use super::super::EXCHANGE_NAME;
        use crypto_market_type::MarketType;
        use crypto_msg_parser::{extract_symbol, extract_timestamp, parse_trade, TradeSide};

        #[test]
        fn spot() {
            let raw_msg = r#"["push.symbol",{"symbol":"BTC_USDT","data":{"deals":[{"t":1616373554541,"p":"57005.89","q":"0.007811","T":1}]}}]"#;
            let trades = &parse_trade(EXCHANGE_NAME, MarketType::Spot, raw_msg).unwrap();

            assert_eq!(trades.len(), 1);
            let trade = &trades[0];

            crate::utils::check_trade_fields(
                EXCHANGE_NAME,
                MarketType::Spot,
                "BTC/USDT".to_string(),
                extract_symbol(EXCHANGE_NAME, MarketType::Spot, raw_msg).unwrap(),
                trade,
                raw_msg,
            );
            assert_eq!(
                1616373554541,
                extract_timestamp(EXCHANGE_NAME, MarketType::Spot, raw_msg)
                    .unwrap()
                    .unwrap()
            );

            assert_eq!(trade.quantity_base, 0.007811);
            assert_eq!(trade.side, TradeSide::Buy);
        }

        #[test]
        fn linear_swap() {
            let raw_msg = r#"{"channel":"push.deal","data":{"M":1,"O":3,"T":2,"p":57602,"t":1616370338806,"v":14},"symbol":"BTC_USDT","ts":1616370338806}"#;
            let trades = &parse_trade(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg).unwrap();

            assert_eq!(trades.len(), 1);
            let trade = &trades[0];

            crate::utils::check_trade_fields(
                EXCHANGE_NAME,
                MarketType::LinearSwap,
                "BTC/USDT".to_string(),
                extract_symbol(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg).unwrap(),
                trade,
                raw_msg,
            );
            assert_eq!(
                1616370338806,
                extract_timestamp(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg)
                    .unwrap()
                    .unwrap()
            );

            assert_eq!(trade.timestamp, 1616370338806);
            assert_eq!(trade.price, 57602.0);
            assert_eq!(trade.quantity_contract, Some(14.0));
            assert_eq!(trade.quantity_base, 0.0001 * 14.0);
            assert_eq!(trade.quantity_quote, 0.0001 * 14.0 * 57602.0);
            assert_eq!(trade.side, TradeSide::Sell);
        }

        #[test]
        fn inverse_swap() {
            let raw_msg = r#"{"channel":"push.deal","data":{"M":1,"O":3,"T":1,"p":57476.5,"t":1616370470356,"v":79},"symbol":"BTC_USD","ts":1616370470356}"#;
            let trades = &parse_trade(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg).unwrap();

            assert_eq!(trades.len(), 1);
            let trade = &trades[0];

            crate::utils::check_trade_fields(
                EXCHANGE_NAME,
                MarketType::InverseSwap,
                "BTC/USD".to_string(),
                extract_symbol(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg).unwrap(),
                trade,
                raw_msg,
            );
            assert_eq!(
                1616370470356,
                extract_timestamp(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg)
                    .unwrap()
                    .unwrap()
            );

            assert_eq!(trade.timestamp, 1616370470356);
            assert_eq!(trade.price, 57476.5);
            assert_eq!(trade.quantity_contract, Some(79.0));
            assert_eq!(trade.quantity_quote, 100.0 * 79.0);
            assert_eq!(trade.quantity_base, 100.0 * 79.0 / 57476.5);
            assert_eq!(trade.side, TradeSide::Buy);
        }
    }

    #[cfg(test)]
    mod l2_event {
        use super::super::EXCHANGE_NAME;
        use chrono::prelude::*;
        use crypto_market_type::MarketType;
        use crypto_msg_parser::{extract_symbol, extract_timestamp, parse_l2};
        use crypto_msg_type::MessageType;

        #[test]
        fn spot_update() {
            let raw_msg = r#"["push.symbol",{"symbol":"BTC_USDT","data":{"bids":[{"p":"38932.19","q":"0.049010","a":"1908.06663"},{"p":"38931.18","q":"0.038220","a":"1487.94969"}],"asks":[{"p":"38941.81","q":"0.000000","a":"0.00000000"},{"p":"38940.71","q":"0.000000","a":"0.00000000"}]}}]"#;
            let received_at = Utc::now().timestamp_millis();
            let orderbook =
                &parse_l2(EXCHANGE_NAME, MarketType::Spot, raw_msg, Some(received_at)).unwrap()[0];

            assert_eq!(orderbook.asks.len(), 2);
            assert_eq!(orderbook.bids.len(), 2);
            assert!(!orderbook.snapshot);

            crate::utils::check_orderbook_fields(
                EXCHANGE_NAME,
                MarketType::Spot,
                MessageType::L2Event,
                "BTC/USDT".to_string(),
                extract_symbol(EXCHANGE_NAME, MarketType::Spot, raw_msg).unwrap(),
                orderbook,
                raw_msg,
            );
            assert_eq!(
                None,
                extract_timestamp(EXCHANGE_NAME, MarketType::Spot, raw_msg,).unwrap()
            );

            assert_eq!(orderbook.bids[0].price, 38932.19);
            assert_eq!(orderbook.bids[0].quantity_base, 0.04901);
            assert_eq!(orderbook.bids[0].quantity_quote, 1908.06663);
        }

        #[test]
        fn linear_swap_update() {
            let raw_msg = r#"{"channel":"push.depth","data":{"asks":[[38704.5,138686,1]],"bids":[],"version":2427341830},"symbol":"BTC_USDT","ts":1622722473816}"#;
            let orderbook =
                &parse_l2(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg, None).unwrap()[0];

            assert_eq!(orderbook.asks.len(), 1);
            assert_eq!(orderbook.bids.len(), 0);
            assert!(!orderbook.snapshot);

            crate::utils::check_orderbook_fields(
                EXCHANGE_NAME,
                MarketType::LinearSwap,
                MessageType::L2Event,
                "BTC/USDT".to_string(),
                extract_symbol(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg).unwrap(),
                orderbook,
                raw_msg,
            );
            assert_eq!(
                1622722473816,
                extract_timestamp(EXCHANGE_NAME, MarketType::LinearSwap, raw_msg)
                    .unwrap()
                    .unwrap()
            );

            assert_eq!(orderbook.timestamp, 1622722473816);
            assert_eq!(orderbook.seq_id, Some(2427341830));

            assert_eq!(orderbook.asks[0].price, 38704.5);
            assert_eq!(orderbook.asks[0].quantity_base, 13.8686);
            assert_eq!(orderbook.asks[0].quantity_quote, 38704.5 * 13.8686);
            assert_eq!(orderbook.asks[0].quantity_contract.unwrap(), 138686.0);
        }

        #[test]
        fn inverse_swap_update() {
            let raw_msg = r#"{"channel":"push.depth","data":{"asks":[[38758.5,4172,2]],"bids":[],"version":1151578213},"symbol":"BTC_USD","ts":1622723010000}"#;
            let orderbook =
                &parse_l2(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg, None).unwrap()[0];

            assert_eq!(orderbook.asks.len(), 1);
            assert_eq!(orderbook.bids.len(), 0);
            assert!(!orderbook.snapshot);

            crate::utils::check_orderbook_fields(
                EXCHANGE_NAME,
                MarketType::InverseSwap,
                MessageType::L2Event,
                "BTC/USD".to_string(),
                extract_symbol(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg).unwrap(),
                orderbook,
                raw_msg,
            );
            assert_eq!(
                1622723010000,
                extract_timestamp(EXCHANGE_NAME, MarketType::InverseSwap, raw_msg)
                    .unwrap()
                    .unwrap()
            );

            assert_eq!(orderbook.timestamp, 1622723010000);
            assert_eq!(orderbook.seq_id, Some(1151578213));

            assert_eq!(orderbook.asks[0].price, 38758.5);
            assert_eq!(orderbook.asks[0].quantity_base, 417200.0 / 38758.5);
            assert_eq!(orderbook.asks[0].quantity_quote, 417200.0);
            assert_eq!(orderbook.asks[0].quantity_contract.unwrap(), 4172.0);
        }
    }
}
