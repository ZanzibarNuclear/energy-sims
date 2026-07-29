//! Lightweight unit helpers (no dimensional analysis library in Stage 1).

use crate::constants::KW_PER_WATT;

/// Convert watts to kilowatts.
#[inline]
pub fn w_to_kw(watts: f64) -> f64 {
    watts * KW_PER_WATT
}

/// Convert kilowatts to watts.
#[inline]
pub fn kw_to_w(kilowatts: f64) -> f64 {
    kilowatts / KW_PER_WATT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_kw() {
        assert!((w_to_kw(kw_to_w(3.5)) - 3.5).abs() < 1e-12);
    }
}
