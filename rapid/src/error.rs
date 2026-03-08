use std::env::VarError;
use thiserror::Error;

/// Errors that can occur when using the Rapid client.
#[derive(Debug, Error)]
pub enum RapidError {
    /// The `RAPIDAPI_KEY` environment variable is not set.
    #[error("missing RAPIDAPI_KEY environment variable: {0}")]
    MissingApiKey(#[from] VarError),

    /// An HTTP or network error from reqwest.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
}

/// Convenience alias for `Result<T, RapidError>`.
pub type Result<T> = std::result::Result<T, RapidError>;
