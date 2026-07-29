//! Errors from pure evaluation and config parsing.

use thiserror::Error;

/// Fallible operations in `energy-sim-core`.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Core evaluation / config errors.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid plant configuration: {0}")]
    InvalidConfig(String),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}
