# crypto-contract-value

[![](https://img.shields.io/crates/v/crypto-contract-value.svg)](https://crates.io/crates/crypto-contract-value)
[![](https://docs.rs/crypto-contract-value/badge.svg)](https://docs.rs/crypto-contract-value)
==========

The value of an unit of contract diffs in different exchanges, and even in the same exchange it differs in different markets.

For example:

- Each Binance perpetual `BTCUSDT` contract is valued at 100 USD, and each alt coin contract is valued at 10 USD.
- Each OKX `BTC-USDT-SWAP` contract is valued at 0.01 BTC.
- Each OKX `BTC-USD-SWAP` contract is valued at 100 USD.
- The contract value of spot markets is always 1.

Given `quantity`, the number of traded coins/contracts, we can multiply it by `contract_value` to get the total traded coins/USDs.
