//! Simulation runtime: sessions, clock, ramps, station grid, and history.
//!
//! Builds on [`energy_sim_core`] for pure evaluation. Stage 1 will add
//! session lifecycle, ramp dynamics, bus balance, and file-oriented exports.

use energy_sim_core::VERSION as CORE_VERSION;

/// Crate version string (from Cargo package metadata).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns a short banner identifying core + runtime versions.
pub fn engine_banner() -> String {
    format!("energy-sim runtime {VERSION} (core {CORE_VERSION})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_mentions_both_crates() {
        let banner = engine_banner();
        assert!(banner.contains("runtime"));
        assert!(banner.contains("core"));
    }
}
