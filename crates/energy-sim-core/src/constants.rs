//! SI defaults and conversion factors.

/// Density of liquid water at ~4 °C (kg/m³). Overridable per plant config.
pub const DEFAULT_RHO_KG_M3: f64 = 1000.0;

/// Standard gravity (m/s²). Overridable per plant config.
pub const DEFAULT_G_MS2: f64 = 9.80665;

/// Watts in one kilowatt.
pub const KW_PER_WATT: f64 = 1.0 / 1000.0;

/// Joules (W·s) in one kilowatt-hour.
pub const WATT_S_PER_KWH: f64 = 3_600_000.0;
