use serde::Deserialize;

/// A single economic calendar event.
#[derive(Debug, Clone, Deserialize)]
pub struct EconEvent {
    pub actual: Option<f64>,
    pub comment: Option<String>,
    pub country: Option<String>,
    pub currency: Option<String>,
    pub date: Option<String>,
    pub forecast: Option<f64>,
    pub id: Option<String>,
    pub importance: Option<i64>,
    pub indicator: Option<String>,
    pub link: Option<String>,
    pub period: Option<String>,
    pub previous: Option<f64>,
    pub scale: Option<String>,
    pub source: Option<String>,
    pub title: Option<String>,
    pub unit: Option<String>,
}

/// Top-level response from the economic calendar endpoint.
#[derive(Debug, Deserialize)]
pub(crate) struct CalendarResponse {
    pub result: Vec<EconEvent>,
}
