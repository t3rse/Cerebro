use serde::Deserialize;

// ── Instruments ───────────────────────────────────────────────────────────────

/// Metadata for a single tradable instrument on the Crypto.com Exchange.
///
/// Returned by [`crate::Hodl::get_instruments`].
#[derive(Debug, Clone, Deserialize)]
pub struct Instrument {
    /// Instrument symbol, e.g. `"BTC_USDT"` or `"BTCUSD-PERP"`.
    pub symbol: Option<String>,
    /// Instrument type, e.g. `"CCY_PAIR"`, `"PERPETUAL_SWAP"`, `"FUTURE"`.
    pub inst_type: Option<String>,
    /// Human-readable display name.
    pub display_name: Option<String>,
    /// Base currency, e.g. `"BTC"`.
    pub base_ccy: Option<String>,
    /// Quote currency, e.g. `"USDT"`.
    pub quote_ccy: Option<String>,
    /// Minimum price increment (tick size).
    pub price_tick_size: Option<String>,
    /// Minimum quantity increment.
    pub qty_tick_size: Option<String>,
    /// Maximum leverage allowed, e.g. `"50"`.
    pub max_leverage: Option<String>,
    /// Whether the instrument is currently tradable.
    pub tradable: Option<bool>,
    /// Expiry timestamp in milliseconds (futures only).
    pub expiry_timestamp_ms: Option<i64>,
    /// Underlying symbol for derivatives.
    pub underlying_symbol: Option<String>,
}

// ── Order book ────────────────────────────────────────────────────────────────

/// A single level in an order book (bid or ask).
///
/// Each level is `[price, size, order_count]`.
#[derive(Debug, Clone, Deserialize)]
pub struct BookLevel {
    /// Price at this level.
    pub price: Option<f64>,
    /// Aggregate size at this price level.
    pub size: Option<f64>,
    /// Number of open orders at this level.
    pub order_count: Option<u64>,
}

/// Order book snapshot for one instrument.
///
/// Returned by [`crate::Hodl::get_book`].
#[derive(Debug, Clone, Deserialize)]
pub struct OrderBook {
    /// Depth of the snapshot.
    pub depth: Option<u32>,
    /// Instrument name.
    pub instrument_name: Option<String>,
    /// Asks (sell side), best ask first.
    pub asks: Option<Vec<BookLevel>>,
    /// Bids (buy side), best bid first.
    pub bids: Option<Vec<BookLevel>>,
}

// ── Candlestick ───────────────────────────────────────────────────────────────

/// A single OHLCV candlestick bar.
///
/// Returned by [`crate::Hodl::get_candlestick`].
#[derive(Debug, Clone, Deserialize)]
pub struct Candle {
    /// Opening price.
    pub open: Option<f64>,
    /// Intra-period high.
    pub high: Option<f64>,
    /// Intra-period low.
    pub low: Option<f64>,
    /// Closing price.
    pub close: Option<f64>,
    /// Volume traded during the period.
    pub volume: Option<f64>,
    /// Start of the candle period, Unix timestamp in milliseconds.
    pub timestamp: Option<i64>,
}

// ── Trades ────────────────────────────────────────────────────────────────────

/// A single public trade.
///
/// Returned by [`crate::Hodl::get_trades`].
#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    /// Trade direction: `"buy"` or `"sell"` (taker side).
    pub side: Option<String>,
    /// Unix timestamp in milliseconds.
    pub timestamp: Option<i64>,
    /// Unique trade identifier.
    pub trade_id: Option<String>,
    /// Quantity traded.
    pub quantity: Option<f64>,
    /// Trade price.
    pub price: Option<f64>,
    /// Instrument name.
    pub instrument_name: Option<String>,
}

// ── Tickers ───────────────────────────────────────────────────────────────────

/// Ticker summary for one instrument.
///
/// Returned by [`crate::Hodl::get_tickers`].
#[derive(Debug, Clone, Deserialize)]
pub struct Ticker {
    /// Instrument name.
    pub instrument_name: Option<String>,
    /// 24-hour high.
    pub h: Option<String>,
    /// 24-hour low.
    pub l: Option<String>,
    /// Best ask price.
    pub a: Option<String>,
    /// Best bid price.
    pub b: Option<String>,
    /// Last traded price.
    pub c: Option<String>,
    /// 24-hour volume in base currency.
    pub v: Option<String>,
    /// 24-hour volume in USD.
    pub vv: Option<String>,
    /// Open interest (derivatives).
    pub oi: Option<String>,
    /// Instrument kind, e.g. `"SPOT"`, `"PERPETUAL_SWAP"`.
    pub k: Option<String>,
    /// Timestamp in milliseconds.
    pub t: Option<i64>,
}

// ── Valuations ────────────────────────────────────────────────────────────────

/// A single valuation data point (index price, mark price, funding rate, etc.).
///
/// Returned by [`crate::Hodl::get_valuations`].
#[derive(Debug, Clone, Deserialize)]
pub struct Valuation {
    /// The valuation value.
    pub v: Option<String>,
    /// Unix timestamp in milliseconds.
    pub t: Option<i64>,
}
