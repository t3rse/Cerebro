//! Thin wrapper around the Ultimate Economic Calendar RapidAPI
//! (`ultimate-economic-calendar.p.rapidapi.com`).
//!
//! All requests require a valid RapidAPI key set in the `RAPID_API_KEY`
//! environment variable.  Construct a [`Rapid`] client once and reuse it.
//!
//! # Example
//!
//! ```no_run
//! use rapid::Rapid;
//!
//! # #[tokio::main]
//! # async fn main() -> rapid::Result<()> {
//! let client = Rapid::new()?;
//! let events = client
//!     .calendar(Some("US"), Some("2024-01-01"), Some("2024-01-31"))
//!     .await?;
//! for e in &events {
//!     println!("{:?}: {:?}", e.date, e.title);
//! }
//! # Ok(())
//! # }
//! ```
pub mod error;
pub mod models;

use std::env;

use reqwest::Client;

pub use error::{RapidError, Result};
pub use models::EconEvent;

const HOST: &str = "ultimate-economic-calendar.p.rapidapi.com";
const BASE_URL: &str =
    "https://ultimate-economic-calendar.p.rapidapi.com/economic-events/tradingview";

/// Client for fetching economic calendar data via the Trading Economics RapidAPI.
///
/// Constructed with [`Rapid::new`], which reads `RAPID_API_KEY` from the
/// environment.  All methods are `async` and return [`Result<T>`].
pub struct Rapid {
    api_key: String,
    client: Client,
}

impl Rapid {
    /// Create a new `Rapid` client.
    ///
    /// Reads the `RAPID_API_KEY` environment variable.  Returns
    /// [`RapidError::MissingApiKey`] if the variable is absent.
    pub fn new() -> Result<Self> {
        let api_key = env::var("RAPID_API_KEY")?;
        Ok(Self {
            api_key,
            client: Client::new(),
        })
    }

    /// Fetch economic calendar events, optionally filtered by country and date range.
    ///
    /// - `country`: ISO country code (e.g. `"US"`, `"GB"`), or `None` for all
    ///   countries.
    /// - `from` / `to`: date strings in `YYYY-MM-DD` format, or `None` for no
    ///   date filter.  Both must be `Some` to apply a date range; if either is
    ///   `None` the date filter is omitted entirely.
    ///
    /// Returns a flat list of [`EconEvent`] values.  All fields on `EconEvent`
    /// are `Option<>` because the upstream API returns sparse objects.
    pub async fn calendar(
        &self,
        country: Option<&str>,
        from: Option<&str>,
        to: Option<&str>,
    ) -> Result<Vec<EconEvent>> {
        let mut url = BASE_URL.to_owned();

        if let Some(c) = country {
            url.push('/');
            url.push_str(c);
        }

        let mut query: Vec<(&str, &str)> = Vec::new();
        if let (Some(f), Some(t)) = (from, to) {
            query.push(("from", f));
            query.push(("to", t));
        }

        let resp = self
            .client
            .get(&url)
            .header("X-RapidAPI-Key", &self.api_key)
            .header("X-RapidAPI-Host", HOST)
            .query(&query)
            .send()
            .await?
            .error_for_status()?;

        let body: models::CalendarResponse = resp.json().await?;
        Ok(body.result)
    }
}
