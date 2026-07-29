//! Pure physics and types for the energy simulation engine.
//!
//! Stage 1 hosts hydro power evaluation, a short head-loss catalog, shared
//! units/constants, and plant configuration types. This crate stays free of
//! session state and I/O beyond JSON deserialization of configs.

pub mod constants;
pub mod error;
pub mod hydro;
pub mod units;

pub use constants::{DEFAULT_G_MS2, DEFAULT_RHO_KG_M3, KW_PER_WATT, WATT_S_PER_KWH};
pub use error::{CoreError, Result};
pub use hydro::{
    evaluate_plant, evaluate_plant_with_inputs, FluidConfig, GeneratorConfig, HydroEvaluation,
    HydroPlantConfig, OperatorInputs, PenstockConfig, StreamConfig, TurbineConfig,
};
pub use units::{kw_to_w, w_to_kw};

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
