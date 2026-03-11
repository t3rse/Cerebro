//! Integration tests for the `ydata` crate.
//!
//! Tests marked `#[ignore]` require an active network connection.  Run them
//! with:
//!
//! ```text
//! cargo test -p ydata -- --include-ignored
//! ```

use time::{Duration, OffsetDateTime};
use ydata::{YData, YDataError};

// ── Unit tests (no network) ───────────────────────────────────────────────────

#[test]
fn result_alias_ok_works() {
    let v: ydata::Result<i32> = Ok(42);
    assert_eq!(v.unwrap(), 42);
}

// ── Integration tests ─────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires network access"]
async fn get_quote_history_returns_bars() {
    let client = YData::new();
    let end = OffsetDateTime::now_utc();
    let start = end - Duration::days(30);
    let bars = client
        .get_quote_history("AAPL", start, end)
        .await
        .expect("get_quote_history failed");
    assert!(!bars.is_empty(), "expected at least one bar");
    for bar in &bars {
        assert!(bar.high >= bar.low, "high should be >= low");
        assert!(bar.open > 0.0, "open should be positive");
        assert!(bar.volume > 0, "volume should be positive");
        assert!(bar.timestamp > 0, "timestamp should be positive");
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn get_quote_history_bars_are_chronological() {
    let client = YData::new();
    let end = OffsetDateTime::now_utc();
    let start = end - Duration::days(30);
    let bars = client
        .get_quote_history("MSFT", start, end)
        .await
        .expect("get_quote_history failed");
    for window in bars.windows(2) {
        assert!(
            window[0].timestamp <= window[1].timestamp,
            "bars should be ordered chronologically"
        );
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn get_quote_history_adjclose_near_close() {
    let client = YData::new();
    let end = OffsetDateTime::now_utc();
    let start = end - Duration::days(30);
    let bars = client
        .get_quote_history("MSFT", start, end)
        .await
        .expect("get_quote_history failed");
    for bar in &bars {
        // adjclose should be within a reasonable multiple of close
        // (large dividends or splits can create bigger divergences over time)
        let ratio = bar.adjclose / bar.close;
        assert!(
            ratio > 0.1 && ratio < 10.0,
            "adjclose/close ratio out of range: {ratio}"
        );
    }
}

#[tokio::test]
#[ignore = "requires network access"]
async fn get_quote_history_invalid_ticker_returns_error() {
    let client = YData::new();
    let end = OffsetDateTime::now_utc();
    let start = end - Duration::days(7);
    let result = client
        .get_quote_history("ZZZZINVALIDTICKER999", start, end)
        .await;
    assert!(result.is_err(), "expected an error for an invalid ticker");
    assert!(
        matches!(result.unwrap_err(), YDataError::Yahoo(_)),
        "error should be YDataError::Yahoo"
    );
}
