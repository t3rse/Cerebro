/// A single historical OHLCV bar for a stock.
#[derive(Debug, Clone)]
pub struct QuoteBar {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adjclose: f64,
    pub volume: u64,
}
