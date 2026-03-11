//! Integration tests for the `headset` crate.
//!
//! Tests marked `#[ignore]` require a valid `FINNHUB_API_KEY` environment
//! variable and an active network connection.  Run them with:
//!
//! ```text
//! cargo test -p headset -- --include-ignored
//! ```

use headset::{Headset, HeadsetError, NewsCategory};

// ── Unit tests (no network) ───────────────────────────────────────────────────

#[test]
fn missing_api_key_error_display() {
    let err = HeadsetError::MissingApiKey(std::env::VarError::NotPresent);
    assert!(err.to_string().contains("FINNHUB_API_KEY"));
}

// ── Integration tests ─────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "requires FINNHUB_API_KEY and network access"]
async fn quote_returns_valid_data() {
    let client = Headset::new().expect("FINNHUB_API_KEY must be set");
    let quote = client.quote("AAPL").await.expect("quote failed");
    assert_eq!(quote.symbol, "AAPL");
    assert!(quote.current_price > 0.0, "price should be positive");
    assert!(quote.high >= quote.low, "high should be >= low");
}

#[tokio::test]
#[ignore = "requires FINNHUB_API_KEY and network access"]
async fn market_news_returns_articles() {
    let client = Headset::new().expect("FINNHUB_API_KEY must be set");
    let articles = client
        .market_news(NewsCategory::General, None)
        .await
        .expect("market_news failed");
    assert!(!articles.is_empty(), "expected at least one article");
    let first = &articles[0];
    assert!(!first.headline.is_empty());
    assert!(!first.url.is_empty());
}

#[tokio::test]
#[ignore = "requires FINNHUB_API_KEY and network access"]
async fn market_news_pagination_with_min_id() {
    let client = Headset::new().expect("FINNHUB_API_KEY must be set");
    let all = client
        .market_news(NewsCategory::General, None)
        .await
        .expect("first fetch failed");
    let max_id = all.iter().map(|a| a.id).max().unwrap_or(0);
    // Fetching with max_id should return no articles already seen.
    let newer = client
        .market_news(NewsCategory::General, Some(max_id))
        .await
        .expect("second fetch failed");
    for a in &newer {
        assert!(a.id > max_id, "article id should exceed min_id");
    }
}

#[tokio::test]
#[ignore = "requires FINNHUB_API_KEY and network access"]
async fn company_news_does_not_error() {
    let client = Headset::new().expect("FINNHUB_API_KEY must be set");
    let articles = client
        .company_news("AAPL", "2024-01-01", "2024-01-31")
        .await
        .expect("company_news failed");
    // The response may be empty on a narrow range / free tier key.
    for a in &articles {
        assert!(!a.headline.is_empty());
    }
}

#[tokio::test]
#[ignore = "requires FINNHUB_API_KEY and network access"]
async fn basic_financials_returns_metrics() {
    let client = Headset::new().expect("FINNHUB_API_KEY must be set");
    let bf = client
        .basic_financials("AAPL")
        .await
        .expect("basic_financials failed");
    assert_eq!(bf.symbol, "AAPL");
    assert!(!bf.metrics.is_empty(), "expected at least one metric");
}

#[tokio::test]
#[ignore = "requires FINNHUB_API_KEY and network access"]
async fn filings_returns_entries() {
    let client = Headset::new().expect("FINNHUB_API_KEY must be set");
    let filings = client.filings("AAPL").await.expect("filings failed");
    assert!(!filings.is_empty(), "expected at least one filing");
    let first = &filings[0];
    assert!(first.form.is_some(), "form type should be present");
    assert!(
        first.report_url.is_some() || first.filing_url.is_some(),
        "at least one URL should be present"
    );
}

#[tokio::test]
#[ignore = "requires FINNHUB_API_KEY and network access"]
async fn company_peers_returns_tickers() {
    let client = Headset::new().expect("FINNHUB_API_KEY must be set");
    let peers = client
        .company_peers("AAPL")
        .await
        .expect("company_peers failed");
    assert!(!peers.is_empty(), "expected at least one peer");
    for p in &peers {
        assert!(!p.is_empty(), "peer ticker should not be empty");
    }
}

#[tokio::test]
#[ignore = "requires FINNHUB_API_KEY and network access"]
async fn earnings_returns_reports_for_date_range() {
    let client = Headset::new().expect("FINNHUB_API_KEY must be set");
    let reports = client
        .earnings(Some("2024-01-01"), Some("2024-01-31"), None)
        .await
        .expect("earnings failed");
    for r in &reports {
        if let Some(sym) = &r.symbol {
            assert!(!sym.is_empty(), "symbol should not be empty");
        }
    }
}

#[tokio::test]
#[ignore = "requires FINNHUB_API_KEY and network access"]
async fn earnings_filtered_by_symbol() {
    let client = Headset::new().expect("FINNHUB_API_KEY must be set");
    let reports = client
        .earnings(None, None, Some("AAPL"))
        .await
        .expect("earnings failed");
    for r in &reports {
        if let Some(sym) = &r.symbol {
            assert_eq!(sym, "AAPL");
        }
    }
}
