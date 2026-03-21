use thiserror::Error;

/// Errors that can occur when using the Hodl client.
#[derive(Debug, Error)]
pub enum HodlError {
    /// An HTTP or network-level error from `reqwest`.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// The API returned a non-zero error code.
    #[error("api error {code}: {message}")]
    Api { code: i64, message: String },
}

/// Convenience alias for `Result<T, HodlError>`.
pub type Result<T> = std::result::Result<T, HodlError>;
