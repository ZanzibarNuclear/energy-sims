//! Steady-state hydro power evaluation.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::hydro::config::HydroPlantConfig;
use crate::hydro::losses::{compute_head_loss_m, LossBreakdown};
use crate::units::w_to_kw;

/// Operator / environment inputs that adjust effective flow and availability.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorInputs {
    /// Gate / admission opening fraction in \[0, 1\]. Scales diverted flow.
    pub gate_opening: f64,
    /// Debris / screen clog fraction of open area in \[0, 1\].
    pub debris_clog_fraction: f64,
    /// Fraction of captured flow lost to leakage before the turbine \[0, 1\].
    pub leakage_fraction: f64,
    /// When false, no power is delivered (plant offline / isolated).
    pub online: bool,
}

impl Default for OperatorInputs {
    fn default() -> Self {
        Self {
            gate_opening: 1.0,
            debris_clog_fraction: 0.0,
            leakage_fraction: 0.0,
            online: true,
        }
    }
}

/// Steady-state evaluation result (targets for the runtime ramp layer).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HydroEvaluation {
    pub plant_id: String,
    /// Effective flow through the turbine (m³/s).
    pub flow_m3s: f64,
    pub gross_head_m: f64,
    pub head_loss_m: f64,
    pub net_head_m: f64,
    pub loss_breakdown: LossBreakdownDto,
    /// Hydraulic power \(\rho g Q H_\mathrm{net}\) (kW).
    pub hydraulic_power_kw: f64,
    /// Electrical power after η_turbine · η_generator (kW), capped at rated.
    pub electrical_power_kw: f64,
    /// Uncapped electrical power before nameplate clip (kW).
    pub electrical_power_uncapped_kw: f64,
    /// Target turbine speed (rpm): design speed when online and flowing, else 0.
    pub turbine_speed_rpm: f64,
    /// True when generation is considered available to the bus.
    pub delivering: bool,
    pub warnings: Vec<String>,
}

/// Serializable loss breakdown (serde-friendly copy of [`super::losses::LossBreakdown`]).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LossBreakdownDto {
    pub friction_m: f64,
    pub minor_m: f64,
    pub debris_m: f64,
    pub total_m: f64,
}

impl From<LossBreakdown> for LossBreakdownDto {
    fn from(b: LossBreakdown) -> Self {
        Self {
            friction_m: b.friction_m,
            minor_m: b.minor_m,
            debris_m: b.debris_m,
            total_m: b.total_m,
        }
    }
}

/// Evaluate steady-state plant power with default full-open operator inputs.
pub fn evaluate_plant(plant: &HydroPlantConfig) -> Result<HydroEvaluation> {
    evaluate_plant_with_inputs(plant, &OperatorInputs::default())
}

/// Evaluate steady-state plant power for the given operator / environment inputs.
///
/// Core equation:
/// \[
/// P_\mathrm{hydraulic} = \rho\, g\, Q\, H_\mathrm{net},\quad
/// H_\mathrm{net} = \max(0, H_\mathrm{gross} - H_\mathrm{loss})
/// \]
/// \[
/// P_\mathrm{electrical} = \eta_t\,\eta_g\, P_\mathrm{hydraulic}
/// \]
pub fn evaluate_plant_with_inputs(
    plant: &HydroPlantConfig,
    inputs: &OperatorInputs,
) -> Result<HydroEvaluation> {
    plant.validate()?;

    let mut warnings = Vec::new();
    let gate = inputs.gate_opening.clamp(0.0, 1.0);
    let leak = inputs.leakage_fraction.clamp(0.0, 1.0);
    let clog = inputs.debris_clog_fraction.clamp(0.0, 1.0);

    // Debris reduces captured flow slightly (area restriction) in addition to head loss.
    let capture = plant.stream.available_flow_m3s * gate * (1.0 - 0.5 * clog);
    let after_leak = capture * (1.0 - leak);

    let mut flow = after_leak;
    if let Some(max_safe) = plant.turbine.max_safe_flow_m3s {
        if flow > max_safe {
            warnings.push(format!(
                "flow capped at maxSafeFlowM3s ({max_safe} m³/s); requested {flow:.6}"
            ));
            flow = max_safe;
        }
    }

    if flow > plant.turbine.design_flow_m3s * 1.05 {
        warnings.push(format!(
            "flow {:.6} m³/s exceeds design flow {:.6} m³/s",
            flow, plant.turbine.design_flow_m3s
        ));
    }

    let losses = compute_head_loss_m(plant, flow, clog);
    let gross = plant.penstock.gross_head_m;
    let net_head = (gross - losses.total_m).max(0.0);
    if losses.total_m > gross && flow > 0.0 {
        warnings.push("head losses exceed gross head; net head clamped to 0".into());
    }

    let rho = plant.fluid.density_kg_m3;
    let g = plant.fluid.gravity_ms2;
    // P_W = ρ g Q H
    let hydraulic_w = rho * g * flow * net_head;
    let hydraulic_kw = w_to_kw(hydraulic_w);

    let eta = plant.turbine.efficiency * plant.generator.efficiency;
    let electrical_uncapped_kw = hydraulic_kw * eta;
    let mut electrical_kw = electrical_uncapped_kw;
    if electrical_kw > plant.generator.rated_power_kw {
        warnings.push(format!(
            "electrical power capped at ratedPowerKw ({})",
            plant.generator.rated_power_kw
        ));
        electrical_kw = plant.generator.rated_power_kw;
    }

    let delivering = inputs.online && flow > 0.0 && electrical_kw > 0.0;
    let electrical_out = if inputs.online { electrical_kw } else { 0.0 };
    let hydraulic_out = if inputs.online { hydraulic_kw } else { 0.0 };

    if !inputs.online {
        warnings.push("plant offline; no power delivered".into());
    }

    let speed = if inputs.online && flow > 0.0 && net_head > 0.0 {
        // Speed scales gently with flow relative to design (teaching model).
        let flow_ratio = if plant.turbine.design_flow_m3s > 0.0 {
            (flow / plant.turbine.design_flow_m3s).clamp(0.0, 1.2)
        } else {
            1.0
        };
        plant.turbine.design_speed_rpm * flow_ratio.min(1.0)
    } else {
        0.0
    };

    Ok(HydroEvaluation {
        plant_id: plant.id.clone(),
        flow_m3s: if inputs.online { flow } else { 0.0 },
        gross_head_m: gross,
        head_loss_m: losses.total_m,
        net_head_m: net_head,
        loss_breakdown: losses.into(),
        hydraulic_power_kw: hydraulic_out,
        electrical_power_kw: electrical_out,
        electrical_power_uncapped_kw: if inputs.online {
            electrical_uncapped_kw
        } else {
            0.0
        },
        turbine_speed_rpm: speed,
        delivering,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_plant() -> HydroPlantConfig {
        HydroPlantConfig::from_json(include_str!(
            "../../../../fixtures/plants/upper-penstock.json"
        ))
        .expect("fixture")
    }

    #[test]
    fn ideal_elevation_only_matches_equation() {
        let mut plant = sample_plant();
        plant.penstock.friction_factor = 0.0;
        plant.penstock.minor_loss_coefficient = 0.0;
        // Use full available flow; disable safe cap interaction by setting high.
        plant.turbine.max_safe_flow_m3s = Some(1.0);
        plant.stream.available_flow_m3s = 0.04;
        plant.generator.rated_power_kw = 100.0;

        let eval = evaluate_plant(&plant).unwrap();
        assert!((eval.net_head_m - 25.0).abs() < 1e-9);
        let expected_hyd_w = 1000.0 * 9.80665 * 0.04 * 25.0;
        let expected_hyd_kw = expected_hyd_w / 1000.0;
        assert!((eval.hydraulic_power_kw - expected_hyd_kw).abs() < 1e-9);
        let expected_el = expected_hyd_kw * 0.75 * 0.92;
        assert!((eval.electrical_power_kw - expected_el).abs() < 1e-9);
    }

    #[test]
    fn losses_reduce_power() {
        let mut plant = sample_plant();
        // Avoid nameplate clip masking the loss effect.
        plant.generator.rated_power_kw = 100.0;
        let mut ideal = plant.clone();
        ideal.penstock.friction_factor = 0.0;
        ideal.penstock.minor_loss_coefficient = 0.0;

        let with_loss = evaluate_plant(&plant).unwrap();
        let no_loss = evaluate_plant(&ideal).unwrap();
        assert!(with_loss.net_head_m < no_loss.net_head_m);
        assert!(with_loss.electrical_power_kw < no_loss.electrical_power_kw);
    }

    #[test]
    fn closed_gate_zero_power() {
        let plant = sample_plant();
        let inputs = OperatorInputs {
            gate_opening: 0.0,
            ..Default::default()
        };
        let eval = evaluate_plant_with_inputs(&plant, &inputs).unwrap();
        assert_eq!(eval.flow_m3s, 0.0);
        assert_eq!(eval.electrical_power_kw, 0.0);
        assert!(!eval.delivering);
    }

    #[test]
    fn offline_zero_delivery() {
        let plant = sample_plant();
        let inputs = OperatorInputs {
            online: false,
            ..Default::default()
        };
        let eval = evaluate_plant_with_inputs(&plant, &inputs).unwrap();
        assert_eq!(eval.electrical_power_kw, 0.0);
        assert!(!eval.delivering);
    }

    #[test]
    fn power_non_negative() {
        let plant = sample_plant();
        let eval = evaluate_plant(&plant).unwrap();
        assert!(eval.hydraulic_power_kw >= 0.0);
        assert!(eval.electrical_power_kw >= 0.0);
        assert!(eval.net_head_m >= 0.0);
    }

    #[test]
    fn sample_fixture_produces_plausible_kw() {
        // Upper Penstock teaching plant: order of a few kW, not MW.
        let plant = sample_plant();
        let eval = evaluate_plant(&plant).unwrap();
        assert!(
            eval.electrical_power_kw > 0.5 && eval.electrical_power_kw < 15.0,
            "got {} kW",
            eval.electrical_power_kw
        );
    }
}
