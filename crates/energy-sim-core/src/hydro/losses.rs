//! Head-loss catalog: named causes with simple engineering coefficients.

use crate::hydro::config::HydroPlantConfig;

/// Breakdown of head loss contributions (meters).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LossBreakdown {
    /// Pipe friction (Darcy–Weisbach style with config friction factor).
    pub friction_m: f64,
    /// Entrance, bends, fittings (minor loss \(K \cdot v^2 / 2g\)).
    pub minor_m: f64,
    /// Extra intake loss from debris / screen clog (fraction of open area).
    pub debris_m: f64,
    /// Sum of all contributions before clamping against gross head.
    pub total_m: f64,
}

/// Mean velocity in the penstock for flow \(Q\) (m/s).
pub fn penstock_velocity_ms(flow_m3s: f64, diameter_m: f64) -> f64 {
    if diameter_m <= 0.0 || flow_m3s <= 0.0 {
        return 0.0;
    }
    let area = std::f64::consts::PI * (diameter_m * 0.5).powi(2);
    flow_m3s / area
}

/// Velocity head \(v^2 / (2g)\) (m).
pub fn velocity_head_m(velocity_ms: f64, gravity_ms2: f64) -> f64 {
    if gravity_ms2 <= 0.0 || velocity_ms <= 0.0 {
        return 0.0;
    }
    (velocity_ms * velocity_ms) / (2.0 * gravity_ms2)
}

/// Compute catalog head losses for a plant at a given turbine flow.
///
/// - **Pipe friction:** \( h_f = f \cdot (L/D) \cdot v^2/(2g) \)
/// - **Minor losses:** \( h_m = K \cdot v^2/(2g) \)
/// - **Debris:** extra minor-like loss scaled by clog fraction of open area
///   (when `debris_clog_fraction` is 0, contribution is 0).
///
/// Leakage is modeled as a flow reduction upstream (see evaluation), not as head.
pub fn compute_head_loss_m(
    plant: &HydroPlantConfig,
    flow_m3s: f64,
    debris_clog_fraction: f64,
) -> LossBreakdown {
    let g = plant.fluid.gravity_ms2;
    let d = plant.penstock.diameter_m;
    let v = penstock_velocity_ms(flow_m3s, d);
    let vh = velocity_head_m(v, g);

    let friction_m = if d > 0.0 {
        plant.penstock.friction_factor * (plant.penstock.length_m / d) * vh
    } else {
        0.0
    };

    let minor_m = plant.penstock.minor_loss_coefficient * vh;

    // Debris: clog fraction maps to an additional entrance-style coefficient.
    // Fully clogged → large extra K (order ~10); open → 0.
    let clog = debris_clog_fraction.clamp(0.0, 1.0);
    let debris_k = 10.0 * clog;
    let debris_m = debris_k * vh;

    let total_m = friction_m + minor_m + debris_m;
    LossBreakdown {
        friction_m,
        minor_m,
        debris_m,
        total_m,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hydro::config::HydroPlantConfig;

    fn tiny_plant() -> HydroPlantConfig {
        HydroPlantConfig::from_json(
            r#"{
              "schemaVersion": 1,
              "kind": "hydro-plant",
              "id": "t",
              "stream": { "availableFlowM3s": 0.04 },
              "penstock": {
                "grossHeadM": 25.0,
                "lengthM": 180.0,
                "diameterM": 0.25,
                "frictionFactor": 0.02,
                "minorLossCoefficient": 0.5
              },
              "turbine": { "efficiency": 0.75, "designFlowM3s": 0.04 },
              "generator": { "efficiency": 0.92, "ratedPowerKw": 8.0 }
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn zero_flow_zero_loss() {
        let p = tiny_plant();
        let loss = compute_head_loss_m(&p, 0.0, 0.0);
        assert_eq!(loss.total_m, 0.0);
    }

    #[test]
    fn ideal_zero_coeffs_zero_loss() {
        let mut p = tiny_plant();
        p.penstock.friction_factor = 0.0;
        p.penstock.minor_loss_coefficient = 0.0;
        let loss = compute_head_loss_m(&p, 0.04, 0.0);
        assert_eq!(loss.total_m, 0.0);
    }

    #[test]
    fn larger_diameter_reduces_friction() {
        let mut small = tiny_plant();
        small.penstock.diameter_m = 0.15;
        let mut large = tiny_plant();
        large.penstock.diameter_m = 0.40;
        let q = 0.04;
        let loss_small = compute_head_loss_m(&small, q, 0.0);
        let loss_large = compute_head_loss_m(&large, q, 0.0);
        assert!(loss_small.friction_m > loss_large.friction_m);
    }

    #[test]
    fn debris_adds_loss() {
        let p = tiny_plant();
        let clean = compute_head_loss_m(&p, 0.04, 0.0);
        let dirty = compute_head_loss_m(&p, 0.04, 0.5);
        assert!(dirty.debris_m > 0.0);
        assert!(dirty.total_m > clean.total_m);
    }
}
