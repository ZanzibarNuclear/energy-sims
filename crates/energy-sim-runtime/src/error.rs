//! Runtime errors.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("core error: {0}")]
    Core(#[from] energy_sim_core::CoreError),

    #[error("invalid session configuration: {0}")]
    InvalidConfig(String),

    #[error("session is not running (phase={0:?}); call start() first")]
    NotRunning(crate::session::SessionPhase),

    #[error("invalid time step: {0}")]
    InvalidTime(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("unknown load id: {0}")]
    UnknownLoad(String),

    #[error("{0}")]
    Other(String),
}
