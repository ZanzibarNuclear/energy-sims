//! Linear ramp: approach a target over configurable up/down durations.

/// Move `actual` toward `target` over `dt_s` seconds.
///
/// When increasing, the full span from the previous actual to the new target
/// would take `ramp_up_s` seconds of continuous approach at constant rate
/// equal to `|target - actual| / ramp` only for the *current gap* — Stage 1
/// uses a simpler model: rate = full-scale travel over ramp time is not known,
/// so we use rate such that closing the **current** gap takes `ramp_*_s`.
/// That yields first-order-like smooth curves when the target is fixed.
///
/// Equivalent formulation used here:
/// `step = min(1, dt / ramp_s) * (target - actual)` with separate ramp times
/// for up vs down. When `ramp_s == 0`, snap to target.
pub fn approach(actual: f64, target: f64, dt_s: f64, ramp_up_s: f64, ramp_down_s: f64) -> f64 {
    if dt_s <= 0.0 {
        return actual;
    }
    let rising = target > actual;
    let ramp = if rising { ramp_up_s } else { ramp_down_s };
    if ramp <= 0.0 {
        return target;
    }
    let alpha = (dt_s / ramp).min(1.0);
    actual + (target - actual) * alpha
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_when_ramp_zero() {
        assert_eq!(approach(0.0, 10.0, 1.0, 0.0, 0.0), 10.0);
    }

    #[test]
    fn reaches_target_in_one_ramp_up() {
        let v = approach(0.0, 100.0, 30.0, 30.0, 45.0);
        assert!((v - 100.0).abs() < 1e-9);
    }

    #[test]
    fn halfway_at_half_ramp() {
        let v = approach(0.0, 100.0, 15.0, 30.0, 45.0);
        assert!((v - 50.0).abs() < 1e-9);
    }

    #[test]
    fn down_uses_down_ramp() {
        // 45 s down ramp, 22.5 s → halfway from 100 to 0
        let v = approach(100.0, 0.0, 22.5, 30.0, 45.0);
        assert!((v - 50.0).abs() < 1e-9);
    }

    #[test]
    fn multi_step_monotonic_up() {
        let mut v = 0.0;
        for _ in 0..10 {
            let next = approach(v, 100.0, 3.0, 30.0, 45.0);
            assert!(next >= v);
            v = next;
        }
        // First-order style: after ~1 ramp time of stepped advances, most of the way there.
        assert!(v > 60.0 && v < 100.0, "got {v}");
        // Full snap after one more full ramp from current.
        let done = approach(v, 100.0, 30.0, 30.0, 45.0);
        assert!((done - 100.0).abs() < 1e-9);
    }
}
