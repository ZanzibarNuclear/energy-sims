//! Headless operator for the energy simulation engine.
//!
//! Stage 1 will accept JSON plant/load configs, run sessions, and write
//! checkpoint / series / event files. For now this binary only prints a banner.

fn main() {
    println!("{}", energy_sim_runtime::engine_banner());
    println!("energy-sims CLI scaffold — see docs/design.md for Stage 1 plan.");
}
