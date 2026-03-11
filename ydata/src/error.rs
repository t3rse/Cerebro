use thiserror::Error;
use yahoo_finance_api::YahooError;

/// Errors that can occur when using the YData client.
#[derive(Debug, Error)]
pub enum YDataError {
    /// An error returned by the Yahoo Finance API.
    #[error("yahoo finance error: {0}")]
    Yahoo(#[from] YahooError),
}

/// Convenience alias for `Result<T, YDataError>`.
pub type Result<T> = std::result::Result<T, YDataError>;
