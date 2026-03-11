use serde::Deserialize;

/// A single economic calendar event.
///
/// All fields are `Option<>` because the upstream API returns sparse objects;
/// not every event will have every field populated.
#[derive(Debug, Clone, Deserialize)]
pub struct EconEvent {
    /// Reported or revised actual value of the indicator.
    pub actual: Option<f64>,
    /// Additional commentary or notes from the source.
    pub comment: Option<String>,
    /// ISO country code the event belongs to (e.g. `"US"`).
    pub country: Option<String>,
    /// Currency relevant to the indicator (e.g. `"USD"`).
    pub currency: Option<String>,
    /// Scheduled release date and time (ISO 8601).
    pub date: Option<String>,
    /// Analyst consensus forecast value.
    pub forecast: Option<f64>,
    /// Unique identifier assigned by the upstream API.
    pub id: Option<String>,
    /// Importance tier: `1` = low, `2` = medium, `3` = high.
    pub importance: Option<i64>,
    /// Short indicator code (e.g. `"NFP"` for Non-Farm Payrolls).
    pub indicator: Option<String>,
    /// URL to the source page for more detail.
    pub link: Option<String>,
    /// Reference period the data covers (e.g. `"Jan 2024"`).
    pub period: Option<String>,
    /// Value from the prior release, for comparison.
    pub previous: Option<f64>,
    /// Unit scale multiplier (e.g. `"K"` for thousands, `"B"` for billions).
    pub scale: Option<String>,
    /// Organisation or agency that publishes this indicator.
    pub source: Option<String>,
    /// Human-readable event title (e.g. `"Initial Jobless Claims"`).
    pub title: Option<String>,
    /// Unit of the reported value (e.g. `"%"`, `"USD"`).
    pub unit: Option<String>,
}

/// Top-level response from the economic calendar endpoint.
#[derive(Debug, Deserialize)]
pub(crate) struct CalendarResponse {
    pub result: Vec<EconEvent>,
}
