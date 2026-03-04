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

    /// Fetch market news for the given `category`.
    /// Pass `min_id` to paginate (only articles with id > min_id are returned).
    pub async fn market_news(
        &self,
        category: NewsCategory,
        min_id: Option<i64>,
    ) -> Result<Vec<MarketNews>> {
        let articles = self.client.news().market_news(category, min_id).await?;
        Ok(articles.into_iter().map(MarketNews::from).collect())
    }

    /// Fetch company-specific news for `symbol` between `from` and `to` (YYYY-MM-DD).
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
    pub async fn basic_financials(&self, symbol: &str) -> Result<BasicFinancials> {
        let bf = self.client.stock().metrics(symbol).await?;
        Ok(BasicFinancials {
            symbol: bf.symbol,
            metrics: bf.metric,
        })
    }

    /// Fetch SEC filings for `symbol`.
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
    pub async fn company_peers(&self, symbol: &str) -> Result<Vec<String>> {
        Ok(self.client.stock().peers(symbol, None).await?)
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
