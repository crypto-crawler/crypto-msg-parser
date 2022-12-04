# crypto-message

[![](https://img.shields.io/crates/v/crypto-message.svg)](https://crates.io/crates/crypto-message)
[![](https://docs.rs/crypto-message/badge.svg)](https://docs.rs/crypto-message)
==========

Unified data structures for all cryptocurrency exchanges.

This library contains all output data types of [`crypto-msg-parser`](https://crates.io/crates/crypto-msg-parser).

The `crypto_message::proto` module contains protobuf messages corresponding to message types in `lib.rs`.

The `crypto_message::compact` module contains compact messages corresponding to message types in `lib.rs`.

**Differences**:

* Message types in `lib.rs` are output data types of `crypto-msg-parser`, and they are designed for JSON and CSV.
* Message types in `crypto_message::proto` are protobuf messages, which are designed for disk storage.
* message types in `crypto_message::compact` are suitable for RPC.

    Messages types in `lib.rs` has string fields such as `exchange`, `symbol`, which causes a lot of memory allocation and copying, so these types are NOT suitable for high-performance processing.

    Message types in `crypto_message::proto` are compact, (1) metadata fields such as `exchange`, `symbol` and `pair` are removed to save disk space, because these fields exist in filenames already, and (2) all float numbers are 32-bit to save more disk space.

    Message types in `crypto_message::compact` are equivalent to message types in `lib.rs`, with `exchange` changed to `enum`, `symbol` and `pair` changed to `u64` hash values.
