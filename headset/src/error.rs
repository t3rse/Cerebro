use std::env::VarError;
use thiserror::Error;

/// Errors that can occur when using the Headset client.
#[derive(Debug, Error)]
pub enum HeadsetError {
    /// The `FINNHUB_API_KEY` environment variable is not set.
    #[error("missing FINNHUB_API_KEY environment variable: {0}")]
    MissingApiKey(#[from] VarError),

    /// An error from the upstream Finnhub API client.
    #[error("finnhub API error: {0}")]
    Finnhub(#[from] finnhub::Error),
}

/// Convenience alias for `Result<T, HeadsetError>`.
pub type Result<T> = std::result::Result<T, HeadsetError>;
