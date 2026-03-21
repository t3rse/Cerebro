//! Integration tests for the `hodl` crate.
//!
//! Two categories of tests are provided:
//!
//! - **Unit tests** (no network): `result_alias_ok_works`, `error_display_api`
//! - **Network tests** (marked `#[ignore]`): require an active internet connection.
//!
//! Run network tests with:
//!
//! ```text
//! cargo test -p hodl -- --include-ignored
//! ```

use hodl::{HodlError, Hodl};

// ── Unit tests (no network) ───────────────────────────────────────────────────

#[test]
fn result_alias_ok_works() {
    let v: hodl::Result<i32> = Ok(42);
    assert_eq!(v.unwrap(), 42);
}

#[test]
fn error_display_api() {
    let e = HodlError::Api {
        code: 10004,
        message: "Not found".to_string(),
    };
    assert!(e.to_string().contains("10004"));
    assert!(e.to_string().contains("Not found"));
}

// ── Integration tests ─────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires network access"]
async fn get_instruments_returns_data() {
    let client = Hodl::new();
    let instruments = client.get_instruments().await.expect("get_instruments failed");
    assert!(!instruments.is_empty(), "expected at least one instrument");
    let btc = instruments.iter().find(|i| {
        i.symbol.as_deref().map(|s| s.contains("BTC")).unwrap_or(false)
    });
    assert!(btc.is_some(), "expected a BTC instrument");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn get_book_returns_bids_and_asks() {
    let client = Hodl::new();
    let book = client
        .get_book("BTC_USDT", 5)
        .await
        .expect("get_book failed");
    let asks = book.asks.as_deref().unwrap_or(&[]);
    let bids = book.bids.as_deref().unwrap_or(&[]);
    assert!(!asks.is_empty(), "expected asks");
    assert!(!bids.is_empty(), "expected bids");
    // Best ask >= best bid
    // if let (Some(best_ask), Some(best_bid)) = (asks[0].price, bids[0].price) {
    //     assert!(best_ask >= best_bid, "best ask should be >= best bid");
    // }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn get_candlestick_returns_candles() {
    let client = Hodl::new();
    let candles = client
        .get_candlestick("BTC_USDT", Some("1h"), Some(10))
        .await
        .expect("get_candlestick failed");
    assert!(!candles.is_empty(), "expected candles");
    for c in &candles {
        if let (Some(high), Some(low)) = (c.high, c.low) {
            assert!(high >= low, "high should be >= low");
        }
        assert!(c.open.unwrap_or(0.0) > 0.0, "open should be positive");
        assert!(c.timestamp.unwrap_or(0) > 0, "timestamp should be positive");
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn get_trades_returns_trades() {
    let client = Hodl::new();
    let trades = client
        .get_trades("BTC_USDT", Some(10))
        .await
        .expect("get_trades failed");
    assert!(!trades.is_empty(), "expected trades");
    for t in &trades {
        assert!(t.price.unwrap_or(0.0) > 0.0, "trade price should be positive");
        assert!(t.quantity.unwrap_or(0.0) > 0.0, "trade quantity should be positive");
        assert!(t.timestamp.unwrap_or(0) > 0, "trade timestamp should be positive");
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn get_tickers_all_returns_many() {
    let client = Hodl::new();
    let tickers = client.get_tickers(None).await.expect("get_tickers failed");
    assert!(tickers.len() > 1, "expected multiple tickers");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn get_tickers_single_returns_one() {
    let client = Hodl::new();
    let tickers = client
        .get_tickers(Some("BTC_USDT"))
        .await
        .expect("get_tickers failed");
    println!("{:?}", tickers);
    assert!(!tickers.is_empty(), "expected at least one ticker");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn get_valuations_index_price() {
    let client = Hodl::new();
    let vals = client
        .get_valuations("BTCUSD-INDEX", "index_price", Some(5))
        .await
        .expect("get_valuations failed");
    assert!(!vals.is_empty(), "expected valuations");
    for v in &vals {
        let price: f64 = v.v.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0.0);
        assert!(price > 0.0, "index price should be positive");
        assert!(v.t.unwrap_or(0) > 0, "timestamp should be positive");
    }
}
