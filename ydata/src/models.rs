/// A single historical OHLCV bar for a stock.
///
/// Returned by [`crate::YData::get_quote_history`].  Bars are daily and
/// ordered chronologically.
#[derive(Debug, Clone)]
pub struct QuoteBar {
    /// Unix timestamp (seconds) for the start of the trading day.
    pub timestamp: i64,
    /// Opening price.
    pub open: f64,
    /// Intra-day high.
    pub high: f64,
    /// Intra-day low.
    pub low: f64,
    /// Closing price.
    pub close: f64,
    /// Dividend- and split-adjusted closing price.
    pub adjclose: f64,
    /// Number of shares traded during the session.
    pub volume: u64,
}
