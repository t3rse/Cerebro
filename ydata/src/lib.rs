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

use std::collections::HashMap;

use time::{OffsetDateTime, Time, UtcOffset};
use yahoo_finance_api::YahooConnector;

pub use error::{Result, YDataError};
pub use models::{MarketSnapshot, QuoteBar};

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

impl MarketSnapshot {
    /// Fetch market data for `tickers` from today's market open (09:30 ET) to now.
    ///
    /// Uses a fixed UTC-4 offset (US Eastern Daylight Time) to determine the
    /// current calendar date and compute the 09:30 open time.  For explicit
    /// control over the window, use [`MarketSnapshot::fetch`].
    ///
    /// # Errors
    ///
    /// Returns [`YDataError::Yahoo`] on any network or ticker error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ydata::{MarketSnapshot, YData};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> ydata::Result<()> {
    /// let client = YData::new();
    /// let snapshot = MarketSnapshot::new(&client, vec!["AAPL", "MSFT"]).await?;
    /// for (ticker, bars) in &snapshot.data {
    ///     println!("{}: {} bars", ticker, bars.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(client: &YData, tickers: Vec<&str>) -> Result<Self> {
        let end = OffsetDateTime::now_utc();
        let et = UtcOffset::from_hms(-4, 0, 0).expect("valid ET offset");
        let today = end.to_offset(et).date();
        let open_time = Time::from_hms(9, 30, 0).expect("valid market open time");
        let start = today.with_time(open_time).assume_offset(et);
        Self::fetch(client, tickers, start, end).await
    }

    /// Fetch market data for `tickers` over an explicit `[start, end]` window.
    ///
    /// # Errors
    ///
    /// Returns [`YDataError::Yahoo`] on any network or ticker error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use time::{Duration, OffsetDateTime};
    /// use ydata::{MarketSnapshot, YData};
    ///
    /// # #[tokio::main]
    /// # async fn main() -> ydata::Result<()> {
    /// let client = YData::new();
    /// let end = OffsetDateTime::now_utc();
    /// let start = end - Duration::days(7);
    /// let snapshot = MarketSnapshot::fetch(&client, vec!["SPY", "QQQ"], start, end).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch(
        client: &YData,
        tickers: Vec<&str>,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Self> {
        let mut data = HashMap::new();
        for ticker in tickers {
            let bars = client.get_quote_history(ticker, start, end).await?;
            data.insert(ticker.to_string(), bars);
        }
        Ok(Self { data, start, end })
    }
}
