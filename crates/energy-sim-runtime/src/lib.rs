//! Simulation runtime: sessions, clock, ramps, station grid, and history.
//!
//! Builds on [`energy_sim_core`] for pure evaluation.

pub mod dynamics;
pub mod error;
pub mod export;
pub mod grid;
pub mod history;
pub mod session;
pub mod snapshot;

pub use dynamics::approach;
pub use error::{Result, RuntimeError};
pub use export::{write_events_jsonl, write_series_csv};
pub use grid::{
    BrownoutConfig, GridBalance, GridStatus, LoadConfig, LoadPriority, StationGrid,
    StationGridConfig,
};
pub use history::{Event, EventKind, Sample};
pub use session::{
    AdvanceReport, Command, HydroOperatorState, Session, SessionConfig, SessionPhase,
};
pub use snapshot::Snapshot;

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
