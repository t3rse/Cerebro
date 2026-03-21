//! Thin async client for the [Crypto.com Exchange public REST API][docs].
//!
//! No API key is required — all endpoints exposed here are unauthenticated.
//! Construct a [`Hodl`] client once and reuse it across calls.
//!
//! [docs]: https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html#reference-and-market-data-api
//!
//! # Example
//!
//! ```no_run
//! use hodl::Hodl;
//!
//! # #[tokio::main]
//! # async fn main() -> hodl::Result<()> {
//! let client = Hodl::new();
//!
//! // All tradable instruments
//! let instruments = client.get_instruments().await?;
//! println!("{} instruments available", instruments.len());
//!
//! // BTC/USDT ticker
//! let tickers = client.get_tickers(Some("BTC_USDT")).await?;
//! if let Some(t) = tickers.first() {
//!     println!("BTC last price: {:?}", t.c);
//! }
//! # Ok(())
//! # }
//! ```
pub mod error;
pub mod models;

pub use error::{HodlError, Result};
pub use models::{BookLevel, Candle, Instrument, OrderBook, Ticker, Trade, Valuation};

use reqwest::Client;
use serde::Deserialize;

const BASE_URL: &str = "https://api.crypto.com/exchange/v1";

// ── Internal response envelope ────────────────────────────────────────────────

#[derive(Deserialize)]
struct Envelope<T> {
    code: i64,
    #[serde(default)]
    message: Option<String>,
    result: Option<T>,
}

impl<T> Envelope<T> {
    fn into_result(self) -> Result<T> {
        if self.code != 0 {
            return Err(HodlError::Api {
                code: self.code,
                message: self.message.unwrap_or_default(),
            });
        }
        Ok(self.result.unwrap_or_else(|| {
            // Should not happen for a successful response, but guard anyway.
            panic!("API returned code 0 but no result")
        }))
    }
}

// ── Intermediate serde shapes ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct InstrumentData {
    data: Vec<Instrument>,
}

#[derive(Deserialize)]
struct RawBook {
    #[serde(default)]
    depth: u32,
    #[serde(default)]
    instrument_name: Option<String>,
    #[serde(default)]
    asks: Vec<[serde_json::Value; 3]>,
    #[serde(default)]
    bids: Vec<[serde_json::Value; 3]>,
}

fn parse_levels(raw: Vec<[serde_json::Value; 3]>) -> Vec<BookLevel> {
    raw.into_iter()
        .map(|arr| BookLevel {
            price: arr[0].as_str().and_then(|s| s.parse().ok()),
            size: arr[1].as_str().and_then(|s| s.parse().ok()),
            order_count: arr[2].as_u64(),
        })
        .collect()
}

#[derive(Deserialize)]
struct BookData {
    #[serde(default)]
    depth: u32,
    #[serde(default)]
    instrument_name: Option<String>,
    data: Vec<RawBook>,
}

#[derive(Deserialize)]
struct RawCandle {
    o: serde_json::Value,
    h: serde_json::Value,
    l: serde_json::Value,
    c: serde_json::Value,
    v: serde_json::Value,
    t: i64,
}

#[derive(Deserialize)]
struct CandleData {
    data: Vec<RawCandle>,
}

fn parse_f64(v: &serde_json::Value) -> f64 {
    match v {
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
        serde_json::Value::String(s) => s.parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

#[derive(Deserialize)]
struct RawTrade {
    pub s: Option<String>,
    pub t: i64,
    pub d: Option<String>,
    pub q: serde_json::Value,
    pub p: serde_json::Value,
    pub i: Option<String>,
}

#[derive(Deserialize)]
struct TradeData {
    data: Vec<RawTrade>,
}

#[derive(Deserialize)]
struct TickerData {
    data: Vec<Ticker>,
}

#[derive(Deserialize)]
struct ValuationData {
    data: Vec<Valuation>,
}

// ── Client ────────────────────────────────────────────────────────────────────

/// Client for the Crypto.com Exchange public REST API.
///
/// Constructed with [`Hodl::new`].  No API key is required.
/// All methods are `async` and return [`Result<T>`].
pub struct Hodl {
    http: Client,
    base_url: String,
}

impl Hodl {
    /// Create a new `Hodl` client using the default Crypto.com Exchange base URL.
    pub fn new() -> Self {
        Self {
            http: Client::new(),
            base_url: BASE_URL.to_string(),
        }
    }

    /// Create a `Hodl` client with a custom base URL.  Useful for testing
    /// against a mock server.
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.into(),
        }
    }

    // ── Reference data ────────────────────────────────────────────────────────

    /// Fetch metadata for all supported instruments.
    ///
    /// # Errors
    ///
    /// Returns [`HodlError::Http`] on network failure or [`HodlError::Api`]
    /// if the exchange returns a non-zero error code.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> hodl::Result<()> {
    /// let client = hodl::Hodl::new();
    /// let instruments = client.get_instruments().await?;
    /// assert!(!instruments.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_instruments(&self) -> Result<Vec<Instrument>> {
        let env: Envelope<InstrumentData> = self
            .http
            .get(format!("{}/public/get-instruments", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(env.into_result()?.data)
    }

    // ── Market data ───────────────────────────────────────────────────────────

    /// Fetch the order book snapshot for `instrument_name` at the given `depth`
    /// (1–50 levels per side).
    ///
    /// # Errors
    ///
    /// Returns [`HodlError::Http`] on network failure or [`HodlError::Api`]
    /// if the exchange returns a non-zero error code.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> hodl::Result<()> {
    /// let client = hodl::Hodl::new();
    /// let book = client.get_book("BTC_USDT", 10).await?;
    /// if let Some(asks) = &book.asks {
    ///     println!("best ask: {:?}", asks[0].price);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_book(&self, instrument_name: &str, depth: u32) -> Result<OrderBook> {
        let env: Envelope<BookData> = self
            .http
            .get(format!("{}/public/get-book", self.base_url))
            .query(&[
                ("instrument_name", instrument_name),
                ("depth", &depth.to_string()),
            ])
            .send()
            .await?
            .json()
            .await?;
        let raw = env.into_result()?.data.into_iter().next().unwrap_or(RawBook {
            depth: 0,
            instrument_name: Some(instrument_name.to_string()),
            asks: vec![],
            bids: vec![],
        });
        Ok(OrderBook {
            depth: Some(raw.depth),
            instrument_name: raw.instrument_name,
            asks: Some(parse_levels(raw.asks)),
            bids: Some(parse_levels(raw.bids)),
        })
    }

    /// Fetch OHLCV candlestick bars for `instrument_name`.
    ///
    /// `timeframe` uses the exchange notation: `"1m"`, `"5m"`, `"15m"`,
    /// `"30m"`, `"1h"`, `"4h"`, `"6h"`, `"12h"`, `"1D"`, `"1W"`, `"1M"`.
    /// Defaults to `"1m"` when `None`.
    ///
    /// `count` controls how many candles are returned (default 25, max 300).
    ///
    /// # Errors
    ///
    /// Returns [`HodlError::Http`] on network failure or [`HodlError::Api`]
    /// if the exchange returns a non-zero error code.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> hodl::Result<()> {
    /// let client = hodl::Hodl::new();
    /// let candles = client.get_candlestick("BTC_USDT", Some("1h"), Some(24)).await?;
    /// println!("{} hourly candles", candles.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_candlestick(
        &self,
        instrument_name: &str,
        timeframe: Option<&str>,
        count: Option<u32>,
    ) -> Result<Vec<Candle>> {
        let mut params = vec![("instrument_name", instrument_name.to_string())];
        if let Some(tf) = timeframe {
            params.push(("timeframe", tf.to_string()));
        }
        if let Some(n) = count {
            params.push(("count", n.to_string()));
        }

        let env: Envelope<CandleData> = self
            .http
            .get(format!("{}/public/get-candlestick", self.base_url))
            .query(&params)
            .send()
            .await?
            .json()
            .await?;

        let candles = env
            .into_result()?
            .data
            .into_iter()
            .map(|r| Candle {
                open: Some(parse_f64(&r.o)),
                high: Some(parse_f64(&r.h)),
                low: Some(parse_f64(&r.l)),
                close: Some(parse_f64(&r.c)),
                volume: Some(parse_f64(&r.v)),
                timestamp: Some(r.t),
            })
            .collect();
        Ok(candles)
    }

    /// Fetch recent public trades for `instrument_name`.
    ///
    /// `count` limits the number of trades returned (default 25, max 150).
    ///
    /// # Errors
    ///
    /// Returns [`HodlError::Http`] on network failure or [`HodlError::Api`]
    /// if the exchange returns a non-zero error code.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> hodl::Result<()> {
    /// let client = hodl::Hodl::new();
    /// let trades = client.get_trades("BTC_USDT", Some(10)).await?;
    /// for t in &trades {
    ///     println!("{:?} @ {:?}", t.side, t.price);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_trades(
        &self,
        instrument_name: &str,
        count: Option<u32>,
    ) -> Result<Vec<Trade>> {
        let mut params = vec![("instrument_name", instrument_name.to_string())];
        if let Some(n) = count {
            params.push(("count", n.to_string()));
        }

        let env: Envelope<TradeData> = self
            .http
            .get(format!("{}/public/get-trades", self.base_url))
            .query(&params)
            .send()
            .await?
            .json()
            .await?;

        let trades = env
            .into_result()?
            .data
            .into_iter()
            .map(|r| Trade {
                side: r.s,
                timestamp: Some(r.t),
                trade_id: r.d,
                quantity: Some(parse_f64(&r.q)),
                price: Some(parse_f64(&r.p)),
                instrument_name: r.i,
            })
            .collect();
        Ok(trades)
    }

    /// Fetch ticker(s).  Pass `Some("BTC_USDT")` for a single instrument or
    /// `None` to retrieve all tickers.
    ///
    /// # Errors
    ///
    /// Returns [`HodlError::Http`] on network failure or [`HodlError::Api`]
    /// if the exchange returns a non-zero error code.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> hodl::Result<()> {
    /// let client = hodl::Hodl::new();
    /// let tickers = client.get_tickers(Some("BTC_USDT")).await?;
    /// println!("{:?}", tickers[0].c); // last price
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_tickers(&self, instrument_name: Option<&str>) -> Result<Vec<Ticker>> {
        let mut req = self
            .http
            .get(format!("{}/public/get-tickers", self.base_url));
        if let Some(name) = instrument_name {
            req = req.query(&[("instrument_name", name)]);
        }

        let env: Envelope<TickerData> = req.send().await?.json().await?;
        Ok(env.into_result()?.data)
    }

    /// Fetch valuation data (index price, mark price, funding rates, etc.) for
    /// `instrument_name`.
    ///
    /// `valuation_type` must be one of: `"index_price"`, `"mark_price"`,
    /// `"funding_hist"`, `"funding_rate"`, `"estimated_funding_rate"`.
    ///
    /// `count` controls how many data points are returned (default 25).
    ///
    /// # Errors
    ///
    /// Returns [`HodlError::Http`] on network failure or [`HodlError::Api`]
    /// if the exchange returns a non-zero error code.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main() -> hodl::Result<()> {
    /// let client = hodl::Hodl::new();
    /// let vals = client.get_valuations("BTCUSD-INDEX", "index_price", Some(10)).await?;
    /// for v in &vals {
    ///     println!("ts={:?} price={:?}", v.t, v.v);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_valuations(
        &self,
        instrument_name: &str,
        valuation_type: &str,
        count: Option<u32>,
    ) -> Result<Vec<Valuation>> {
        let mut params = vec![
            ("instrument_name", instrument_name.to_string()),
            ("valuation_type", valuation_type.to_string()),
        ];
        if let Some(n) = count {
            params.push(("count", n.to_string()));
        }

        let env: Envelope<ValuationData> = self
            .http
            .get(format!("{}/public/get-valuations", self.base_url))
            .query(&params)
            .send()
            .await?
            .json()
            .await?;
        Ok(env.into_result()?.data)
    }
}

impl Default for Hodl {
    fn default() -> Self {
        Self::new()
    }
}
