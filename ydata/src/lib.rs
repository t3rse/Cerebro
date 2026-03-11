pub mod error;
pub mod models;

use time::OffsetDateTime;
use yahoo_finance_api::YahooConnector;

pub use error::{YDataError, Result};
pub use models::QuoteBar;

/// Client for fetching historical market data via the Yahoo Finance API.
pub struct YData {
    connector: YahooConnector,
}

impl YData {
    /// Create a new `YData` client.
    pub fn new() -> Self {
        Self {
            connector: YahooConnector::new().expect("failed to create YahooConnector"),
        }
    }

    /// Fetch daily quote history for `ticker` between `start` and `end` (inclusive).
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
