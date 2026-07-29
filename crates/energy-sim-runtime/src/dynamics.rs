//! Fixed-duration S-curve ramps (smoothstep) toward steady-state targets.
//!
//! When a target changes, the actual value follows an ease-in / ease-out curve
//! from the value at retarget time to the new target over `ramp_up_s` or
//! `ramp_down_s` seconds. At the end of the duration, actual equals target
//! exactly (unlike a first-order lag, which only approaches asymptotically).

use serde::{Deserialize, Serialize};

/// Hermite smoothstep on \[0, 1\]: \(S(u) = 3u^2 - 2u^3\).
///
/// Zero derivative at both ends → soft start and soft landing (S-shaped).
#[inline]
pub fn smoothstep(u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// One continuous transition segment for a single scalar quantity.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RampSegment {
    /// Value when this segment started.
    pub start: f64,
    /// Target value at the end of the segment.
    pub target: f64,
    /// Simulation time (s) when the segment started.
    pub t_start_s: f64,
    /// Total duration (s). Zero means snap to target.
    pub duration_s: f64,
}

impl RampSegment {
    /// Instantaneous segment that already sits on `value`.
    pub fn settled(value: f64, sim_time_s: f64) -> Self {
        Self {
            start: value,
            target: value,
            t_start_s: sim_time_s,
            duration_s: 0.0,
        }
    }

    /// Evaluate the S-curve at absolute simulation time `sim_time_s`.
    pub fn value_at(&self, sim_time_s: f64) -> f64 {
        if self.duration_s <= 0.0 {
            return self.target;
        }
        let u = ((sim_time_s - self.t_start_s) / self.duration_s).clamp(0.0, 1.0);
        self.start + (self.target - self.start) * smoothstep(u)
    }

    pub fn is_complete(&self, sim_time_s: f64) -> bool {
        self.duration_s <= 0.0 || sim_time_s >= self.t_start_s + self.duration_s
    }
}

/// If `new_target` differs from the active segment target, start a new S-curve
/// from `current_actual` over the appropriate up/down duration.
///
/// Call **before** advancing time for this step, with `sim_time_s` = time at
/// the start of the step (when the command/target is known).
pub fn ensure_ramp(
    segment: &mut RampSegment,
    current_actual: f64,
    new_target: f64,
    sim_time_s: f64,
    ramp_up_s: f64,
    ramp_down_s: f64,
) {
    const EPS: f64 = 1e-9;
    if (segment.target - new_target).abs() <= EPS {
        return;
    }
    if (new_target - current_actual).abs() <= EPS {
        *segment = RampSegment::settled(new_target, sim_time_s);
        return;
    }
    let rising = new_target > current_actual;
    let duration = if rising {
        ramp_up_s.max(0.0)
    } else {
        ramp_down_s.max(0.0)
    };
    *segment = RampSegment {
        start: current_actual,
        target: new_target,
        t_start_s: sim_time_s,
        duration_s: duration,
    };
}

/// Stateless helper kept for simple one-shot evaluation of an S-curve fraction.
/// Prefer [`RampSegment`] + [`ensure_ramp`] in the session.
pub fn approach(actual: f64, target: f64, dt_s: f64, ramp_up_s: f64, ramp_down_s: f64) -> f64 {
    if dt_s <= 0.0 {
        return actual;
    }
    let rising = target > actual;
    let ramp = if rising { ramp_up_s } else { ramp_down_s };
    if ramp <= 0.0 {
        return target;
    }
    // One-shot: interpret as progress over `ramp` from `actual` toward `target`.
    let mut seg = RampSegment {
        start: actual,
        target,
        t_start_s: 0.0,
        duration_s: ramp,
    };
    ensure_ramp(&mut seg, actual, target, 0.0, ramp_up_s, ramp_down_s);
    seg.value_at(dt_s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoothstep_endpoints() {
        assert!((smoothstep(0.0) - 0.0).abs() < 1e-12);
        assert!((smoothstep(1.0) - 1.0).abs() < 1e-12);
        assert!((smoothstep(0.5) - 0.5).abs() < 1e-12);
        // Mid-slope steeper than linear at ends: S(0.25) < 0.25, S(0.75) > 0.75
        assert!(smoothstep(0.25) < 0.25);
        assert!(smoothstep(0.75) > 0.75);
    }

    #[test]
    fn reaches_target_exactly_at_duration() {
        let seg = RampSegment {
            start: 0.0,
            target: 100.0,
            t_start_s: 0.0,
            duration_s: 20.0,
        };
        assert!((seg.value_at(20.0) - 100.0).abs() < 1e-12);
        assert!((seg.value_at(25.0) - 100.0).abs() < 1e-12);
        assert!(seg.value_at(10.0) > 0.0 && seg.value_at(10.0) < 100.0);
    }

    #[test]
    fn zero_duration_snaps() {
        let seg = RampSegment::settled(7.0, 1.0);
        assert_eq!(seg.value_at(1.0), 7.0);
        assert_eq!(seg.value_at(100.0), 7.0);
    }

    #[test]
    fn ensure_ramp_retargets_on_change() {
        let mut seg = RampSegment::settled(0.0, 0.0);
        ensure_ramp(&mut seg, 0.0, 8.0, 0.0, 20.0, 25.0);
        assert_eq!(seg.target, 8.0);
        assert_eq!(seg.duration_s, 20.0);
        let mid = seg.value_at(10.0);
        // Same target → no restart
        ensure_ramp(&mut seg, mid, 8.0, 10.0, 20.0, 25.0);
        assert_eq!(seg.t_start_s, 0.0);
        // New target → restart from current
        ensure_ramp(&mut seg, mid, 0.0, 10.0, 20.0, 25.0);
        assert_eq!(seg.start, mid);
        assert_eq!(seg.target, 0.0);
        assert_eq!(seg.duration_s, 25.0);
        assert_eq!(seg.t_start_s, 10.0);
    }

    #[test]
    fn multi_step_hits_target_at_20s() {
        let mut seg = RampSegment::settled(0.0, 0.0);
        let mut actual = 0.0;
        let mut t = 0.0;
        ensure_ramp(&mut seg, actual, 100.0, t, 20.0, 25.0);
        for _ in 0..20 {
            t += 1.0;
            actual = seg.value_at(t);
        }
        assert!(
            (actual - 100.0).abs() < 1e-9,
            "expected full power at 20s, got {actual}"
        );
    }

    #[test]
    fn approach_one_shot_reaches_at_full_duration() {
        let v = approach(0.0, 100.0, 20.0, 20.0, 25.0);
        assert!((v - 100.0).abs() < 1e-9);
    }
}
