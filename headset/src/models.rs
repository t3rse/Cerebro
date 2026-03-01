use finnhub::models::calendar::EarningsRelease;
use finnhub::models::news::CompanyNews as FinnhubCompanyNews;
use finnhub::models::news::MarketNews as FinnhubMarketNews;
use finnhub::models::stock::Quote;

/// A real-time stock quote.
#[derive(Debug, Clone)]
pub struct StockQuote {
    pub symbol: String,
    pub current_price: f64,
    pub change: f64,
    pub percent_change: f64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub previous_close: f64,
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

/// A single market news article.
#[derive(Debug, Clone)]
pub struct MarketNews {
    pub category: String,
    pub datetime: i64,
    pub headline: String,
    pub id: i64,
    pub image: String,
    pub related: String,
    pub source: String,
    pub summary: String,
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
    pub category: String,
    pub datetime: i64,
    pub headline: String,
    pub id: i64,
    pub image: String,
    pub related: String,
    pub source: String,
    pub summary: String,
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

/// A single earnings calendar entry.
#[derive(Debug, Clone)]
pub struct EarningsReport {
    pub symbol: Option<String>,
    pub date: Option<String>,
    pub hour: Option<String>,
    pub year: Option<i64>,
    pub quarter: Option<i64>,
    pub eps_estimate: Option<f64>,
    pub eps_actual: Option<f64>,
    pub revenue_estimate: Option<f64>,
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
