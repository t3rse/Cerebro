//! Thin wrapper around the [`finnhub`] crate providing clean domain types for
//! the Cerebro TUI application.
//!
//! All API calls require a valid Finnhub API key set in the `FINNHUB_API_KEY`
//! environment variable.  Construct a [`Headset`] client once and reuse it
//! across `await` calls.
//!
//! # Example
//!
//! ```no_run
//! use headset::Headset;
//!
//! # #[tokio::main]
//! # async fn main() -> headset::Result<()> {
//! let client = Headset::new()?;
//! let quote = client.quote("AAPL").await?;
//! println!("{}: ${:.2} ({:+.2}%)", quote.symbol, quote.current_price, quote.percent_change);
//! # Ok(())
//! # }
//! ```
pub mod error;
pub mod models;

use std::env;

use finnhub::FinnhubClient;

pub use error::{HeadsetError, Result};
pub use finnhub::models::news::NewsCategory;
pub use models::{
    BasicFinancials, CompanyNews, EarningsReport, FilingEntry, MarketNews, StockQuote,
};

/// Client for fetching financial data via the Finnhub API.
///
/// Constructed with [`Headset::new`], which reads `FINNHUB_API_KEY` from the
/// environment.  All methods are `async` and return [`Result<T>`].
pub struct Headset {
    client: FinnhubClient,
}

impl Headset {
    /// Create a new `Headset` client.
    ///
    /// Reads the `FINNHUB_API_KEY` environment variable.  Returns
    /// [`HeadsetError::MissingApiKey`] if the variable is absent.
    pub fn new() -> Result<Self> {
        let api_key = env::var("FINNHUB_API_KEY")?;
        Ok(Self {
            client: FinnhubClient::new(api_key),
        })
    }

    /// Fetch a real-time quote for `symbol` (e.g. `"AAPL"`).
    ///
    /// Returns a [`StockQuote`] with the current price, intra-day range,
    /// change, and percent change relative to the previous close.
    pub async fn quote(&self, symbol: &str) -> Result<StockQuote> {
        let q = self.client.stock().quote(symbol).await?;
        Ok(StockQuote::from_finnhub(symbol, q))
    }

    /// Fetch market-wide news for the given `category`.
    ///
    /// Pass `min_id` to paginate: only articles whose id is strictly greater
    /// than `min_id` are returned.  Pass `None` to retrieve the latest batch.
    pub async fn market_news(
        &self,
        category: NewsCategory,
        min_id: Option<i64>,
    ) -> Result<Vec<MarketNews>> {
        let articles = self.client.news().market_news(category, min_id).await?;
        Ok(articles.into_iter().map(MarketNews::from).collect())
    }

    /// Fetch company-specific news for `symbol` between `from` and `to`.
    ///
    /// Both dates must be in `YYYY-MM-DD` format.
    pub async fn company_news(
        &self,
        symbol: &str,
        from: &str,
        to: &str,
    ) -> Result<Vec<CompanyNews>> {
        let articles = self.client.news().company_news(symbol, from, to).await?;
        Ok(articles.into_iter().map(CompanyNews::from).collect())
    }

    /// Fetch key financial metrics for `symbol`.
    ///
    /// The returned [`BasicFinancials`] contains a map of metric names to
    /// JSON values as reported by Finnhub (e.g. `"52WeekHigh"`, `"peBasicExclExtraTTM"`).
    pub async fn basic_financials(&self, symbol: &str) -> Result<BasicFinancials> {
        let bf = self.client.stock().metrics(symbol).await?;
        Ok(BasicFinancials {
            symbol: bf.symbol,
            metrics: bf.metric,
        })
    }

    /// Fetch SEC filings for `symbol`, ordered most-recent first.
    ///
    /// Each [`FilingEntry`] carries the form type (e.g. `"10-K"`), the filed
    /// date, and URLs pointing to the report and the raw filing on EDGAR.
    pub async fn filings(&self, symbol: &str) -> Result<Vec<FilingEntry>> {
        let entries = self
            .client
            .stock()
            .sec_filings(Some(symbol), None, None, None, None, None)
            .await?;
        Ok(entries
            .into_iter()
            .map(|f| FilingEntry {
                form: f.form,
                filed_date: f.filed_date,
                report_url: f.report_url,
                filing_url: f.filing_url,
            })
            .collect())
    }

    /// Fetch peer company symbols for `symbol`.
    ///
    /// Peers are companies in the same industry/sector as reported by Finnhub.
    pub async fn company_peers(&self, symbol: &str) -> Result<Vec<String>> {
        Ok(self.client.stock().peers(symbol, None).await?)
    }

    /// Fetch earnings calendar entries.
    ///
    /// All parameters are optional:
    /// - `from` / `to`: date range in `YYYY-MM-DD` format.
    /// - `symbol`: filter to a single ticker (e.g. `Some("AAPL")`).
    ///
    /// Passing all `None` returns upcoming earnings across the market.
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
