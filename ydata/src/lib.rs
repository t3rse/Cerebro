//! Thin wrapper around the [`yahoo_finance_api`] crate for fetching historical
//! OHLCV (open/high/low/close/volume) market data.
//!
//! No API key is required — the Yahoo Finance endpoint is unauthenticated.
//! Construct a [`YData`] client once and reuse it across calls.
//!
//! # Example
//!
//! ```no_run
//! use time::{Duration, OffsetDateTime};
//! use ydata::YData;
//!
//! # #[tokio::main]
//! # async fn main() -> ydata::Result<()> {
//! let client = YData::new();
//! let end = OffsetDateTime::now_utc();
//! let start = end - Duration::days(30);
//! let bars = client.get_quote_history("AAPL", start, end).await?;
//! for bar in &bars {
//!     println!("ts={} close={:.2}", bar.timestamp, bar.close);
//! }
//! # Ok(())
//! # }
//! ```
pub mod error;
pub mod models;

use time::OffsetDateTime;
use yahoo_finance_api::YahooConnector;

pub use error::{YDataError, Result};
pub use models::QuoteBar;

/// Client for fetching historical market data via the Yahoo Finance API.
///
/// Constructed with [`YData::new`].  No API key is required.
/// All methods are `async` and return [`Result<T>`].
pub struct YData {
    connector: YahooConnector,
}

impl YData {
    /// Create a new `YData` client.
    ///
    /// Panics if the underlying [`YahooConnector`] cannot be initialised
    /// (this only occurs if the TLS backend fails to load, which is
    /// effectively never in a standard environment).
    pub fn new() -> Self {
        Self {
            connector: YahooConnector::new().expect("failed to create YahooConnector"),
        }
    }

    /// Fetch daily OHLCV bars for `ticker` between `start` and `end` (inclusive).
    ///
    /// `start` and `end` are [`OffsetDateTime`] values from the [`time`] crate.
    /// The returned [`QuoteBar`] slice is ordered chronologically.
    ///
    /// # Errors
    ///
    /// Returns [`YDataError::Yahoo`] if the ticker is unknown, the date range
    /// is invalid, or a network error occurs.
    pub async fn get_quote_history(
        &self,
        ticker: &str,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<QuoteBar>> {
        let response = self.connector.get_quote_history(ticker, start, end).await?;
        let bars = response
            .quotes()?
            .into_iter()
            .map(|q| QuoteBar {
                timestamp: q.timestamp as i64,
                open: q.open,
                high: q.high,
                low: q.low,
                close: q.close,
                adjclose: q.adjclose,
                volume: q.volume,
            })
            .collect();
        Ok(bars)
    }
}
