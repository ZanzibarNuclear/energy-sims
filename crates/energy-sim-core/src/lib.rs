//! Pure physics and types for the energy simulation engine.
//!
//! Stage 1 will host hydro power evaluation, loss models, shared units,
//! and telemetry/event types. This crate stays free of session state and I/O.

/// Crate version string (from Cargo package metadata).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_nonempty() {
        assert!(!VERSION.is_empty());
    }
}
