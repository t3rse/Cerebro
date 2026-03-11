use std::collections::HashMap;

use finnhub::models::calendar::EarningsRelease;
use finnhub::models::news::CompanyNews as FinnhubCompanyNews;
use finnhub::models::news::MarketNews as FinnhubMarketNews;
use finnhub::models::stock::Quote;

/// A real-time stock quote.
#[derive(Debug, Clone)]
pub struct StockQuote {
    /// Ticker symbol in uppercase (e.g. `"AAPL"`).
    pub symbol: String,
    /// Current trading price.
    pub current_price: f64,
    /// Absolute price change from the previous close.
    pub change: f64,
    /// Percent price change from the previous close.
    pub percent_change: f64,
    /// Intra-day high.
    pub high: f64,
    /// Intra-day low.
    pub low: f64,
    /// Opening price for the current session.
    pub open: f64,
    /// Previous session's closing price.
    pub previous_close: f64,
    /// Unix timestamp of the quote.
    pub timestamp: i64,
}

impl StockQuote {
    /// Build a `StockQuote` from a finnhub [`Quote`] and the requested symbol.
    pub fn from_finnhub(symbol: &str, q: Quote) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            current_price: q.current_price,
            change: q.change,
            percent_change: q.percent_change,
            high: q.high,
            low: q.low,
            open: q.open,
            previous_close: q.previous_close,
            timestamp: q.timestamp,
        }
    }
}

/// A single market-wide news article.
#[derive(Debug, Clone)]
pub struct MarketNews {
    /// Finnhub news category (e.g. `"general"`, `"forex"`).
    pub category: String,
    /// Publication time as a Unix timestamp.
    pub datetime: i64,
    /// Article headline.
    pub headline: String,
    /// Unique Finnhub article ID.
    pub id: i64,
    /// URL of the article's thumbnail image.
    pub image: String,
    /// Ticker or topic the article relates to.
    pub related: String,
    /// Name of the publishing outlet.
    pub source: String,
    /// Short summary of the article.
    pub summary: String,
    /// Canonical URL of the article.
    pub url: String,
}

impl From<FinnhubMarketNews> for MarketNews {
    fn from(n: FinnhubMarketNews) -> Self {
        Self {
            category: n.category,
            datetime: n.datetime,
            headline: n.headline,
            id: n.id,
            image: n.image,
            related: n.related,
            source: n.source,
            summary: n.summary,
            url: n.url,
        }
    }
}

/// A company-specific news article.
#[derive(Debug, Clone)]
pub struct CompanyNews {
    /// Finnhub news category.
    pub category: String,
    /// Publication time as a Unix timestamp.
    pub datetime: i64,
    /// Article headline.
    pub headline: String,
    /// Unique Finnhub article ID.
    pub id: i64,
    /// URL of the article's thumbnail image.
    pub image: String,
    /// Ticker the article relates to.
    pub related: String,
    /// Name of the publishing outlet.
    pub source: String,
    /// Short summary of the article.
    pub summary: String,
    /// Canonical URL of the article.
    pub url: String,
}

impl From<FinnhubCompanyNews> for CompanyNews {
    fn from(n: FinnhubCompanyNews) -> Self {
        Self {
            category: n.category,
            datetime: n.datetime,
            headline: n.headline,
            id: n.id,
            image: n.image,
            related: n.related,
            source: n.source,
            summary: n.summary,
            url: n.url,
        }
    }
}

/// Key financial metrics for a stock.
pub struct BasicFinancials {
    /// Ticker symbol the metrics belong to.
    pub symbol: String,
    /// Map of metric name to value as returned by Finnhub
    /// (e.g. `"52WeekHigh"`, `"peBasicExclExtraTTM"`).
    pub metrics: HashMap<String, serde_json::Value>,
}

/// A single SEC filing entry.
pub struct FilingEntry {
    /// Form type, e.g. `"10-K"` or `"8-K"`.
    pub form: Option<String>,
    /// Date the filing was submitted, in `YYYY-MM-DD` format.
    pub filed_date: Option<String>,
    /// Direct URL to the human-readable report on SEC EDGAR.
    pub report_url: Option<String>,
    /// URL to the raw filing index on SEC EDGAR.
    pub filing_url: Option<String>,
}

/// A single earnings calendar entry.
#[derive(Debug, Clone)]
pub struct EarningsReport {
    /// Ticker symbol.
    pub symbol: Option<String>,
    /// Expected or actual report date in `YYYY-MM-DD` format.
    pub date: Option<String>,
    /// Reporting hour: `"bmo"` (before market open) or `"amc"` (after market close).
    pub hour: Option<String>,
    /// Fiscal year the report covers.
    pub year: Option<i64>,
    /// Fiscal quarter (1–4).
    pub quarter: Option<i64>,
    /// Analyst consensus EPS estimate.
    pub eps_estimate: Option<f64>,
    /// Reported EPS (populated after the release).
    pub eps_actual: Option<f64>,
    /// Analyst consensus revenue estimate in USD.
    pub revenue_estimate: Option<f64>,
    /// Reported revenue in USD (populated after the release).
    pub revenue_actual: Option<f64>,
}

impl From<EarningsRelease> for EarningsReport {
    fn from(e: EarningsRelease) -> Self {
        Self {
            symbol: e.symbol,
            date: e.date,
            hour: e.hour,
            year: e.year,
            quarter: e.quarter,
            eps_estimate: e.eps_estimate,
            eps_actual: e.eps_actual,
            revenue_estimate: e.revenue_estimate,
            revenue_actual: e.revenue_actual,
        }
    }
}
