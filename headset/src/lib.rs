pub mod error;
pub mod models;

use std::env;

use finnhub::FinnhubClient;

pub use error::{HeadsetError, Result};
pub use models::{EarningsReport, StockQuote};

/// Client for fetching financial data via the Finnhub API.
pub struct Headset {
    client: FinnhubClient,
}

impl Headset {
    /// Create a new `Headset` by reading the `FINNHUB_API_KEY` environment variable.
    pub fn new() -> Result<Self> {
        let api_key = env::var("FINNHUB_API_KEY")?;
        Ok(Self {
            client: FinnhubClient::new(api_key),
        })
    }

    /// Fetch a real-time quote for `symbol` (e.g. `"AAPL"`).
    pub async fn quote(&self, symbol: &str) -> Result<StockQuote> {
        let q = self.client.stock().quote(symbol).await?;
        Ok(StockQuote::from_finnhub(symbol, q))
    }

    /// Fetch earnings calendar entries, optionally filtered by date range and symbol.
    pub async fn earnings(
        &self,
        from: Option<&str>,
        to: Option<&str>,
        symbol: Option<&str>,
    ) -> Result<Vec<EarningsReport>> {
        let cal = self.client.calendar().earnings(from, to, symbol).await?;
        let reports = cal
            .earnings_calendar
            .into_iter()
            .map(EarningsReport::from)
            .collect();
        Ok(reports)
    }
}
