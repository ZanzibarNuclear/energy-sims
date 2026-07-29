//! JSON-facing hydro plant configuration (field-based Stage 1).

use serde::{Deserialize, Serialize};

use crate::constants::{DEFAULT_G_MS2, DEFAULT_RHO_KG_M3};
use crate::error::{CoreError, Result};

/// Top-level hydro plant document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HydroPlantConfig {
    pub schema_version: u32,
    pub kind: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub stream: StreamConfig,
    pub penstock: PenstockConfig,
    pub turbine: TurbineConfig,
    pub generator: GeneratorConfig,
    #[serde(default)]
    pub fluid: FluidConfig,
    /// Optional catalog hook; ignored until a package resolver exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
}

/// Available stream / intake flow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamConfig {
    /// Available stream flow at intake (m³/s).
    pub available_flow_m3s: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
}

/// Penstock geometry and friction / minor loss coefficients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PenstockConfig {
    /// Gross elevation drop intake → turbine (m).
    pub gross_head_m: f64,
    /// Penstock length (m).
    pub length_m: f64,
    /// Internal diameter (m).
    pub diameter_m: f64,
    /// Darcy friction factor \(f\) (dimensionless), typical 0.015–0.03.
    #[serde(default = "default_friction_factor")]
    pub friction_factor: f64,
    /// Combined minor-loss coefficient \(K\) for entrance, bends, fittings.
    #[serde(default = "default_minor_loss")]
    pub minor_loss_coefficient: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
}

fn default_friction_factor() -> f64 {
    0.02
}

fn default_minor_loss() -> f64 {
    0.5
}

/// Turbine efficiencies and flow limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurbineConfig {
    /// Turbine efficiency η_turbine (0–1).
    pub efficiency: f64,
    /// Design / rated flow (m³/s).
    pub design_flow_m3s: f64,
    /// Maximum safe flow (m³/s); effective flow is capped at this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_safe_flow_m3s: Option<f64>,
    /// Design rotational speed (rpm) used as the steady-state speed target.
    #[serde(default = "default_design_speed_rpm")]
    pub design_speed_rpm: f64,
    /// Ramp times for speed and power (runtime uses these).
    #[serde(default)]
    pub dynamics: TurbineDynamics,
    /// Optional catalog package id (ignored in Stage 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
}

/// S-curve ramp durations (seconds) from current actual to a new steady-state target.
///
/// Runtime uses a smoothstep curve fitted to these durations: at the end of
/// `*RampUpS` / `*RampDownS`, actual equals target exactly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurbineDynamics {
    #[serde(default = "default_speed_ramp_up")]
    pub speed_ramp_up_s: f64,
    #[serde(default = "default_speed_ramp_down")]
    pub speed_ramp_down_s: f64,
    #[serde(default = "default_power_ramp_up")]
    pub power_ramp_up_s: f64,
    #[serde(default = "default_power_ramp_down")]
    pub power_ramp_down_s: f64,
}

impl Default for TurbineDynamics {
    fn default() -> Self {
        Self {
            speed_ramp_up_s: default_speed_ramp_up(),
            speed_ramp_down_s: default_speed_ramp_down(),
            power_ramp_up_s: default_power_ramp_up(),
            power_ramp_down_s: default_power_ramp_down(),
        }
    }
}

fn default_speed_ramp_up() -> f64 {
    20.0
}
fn default_speed_ramp_down() -> f64 {
    25.0
}
fn default_power_ramp_up() -> f64 {
    20.0
}
fn default_power_ramp_down() -> f64 {
    25.0
}

fn default_design_speed_rpm() -> f64 {
    1000.0
}

/// Generator efficiency and nameplate rating.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorConfig {
    /// Generator efficiency η_generator (0–1).
    pub efficiency: f64,
    /// Nameplate / rated electrical power (kW).
    pub rated_power_kw: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
}

/// Working fluid properties (defaults = water on Earth).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FluidConfig {
    #[serde(default = "default_rho")]
    pub density_kg_m3: f64,
    #[serde(default = "default_g")]
    pub gravity_ms2: f64,
}

fn default_rho() -> f64 {
    DEFAULT_RHO_KG_M3
}

fn default_g() -> f64 {
    DEFAULT_G_MS2
}

impl Default for FluidConfig {
    fn default() -> Self {
        Self {
            density_kg_m3: DEFAULT_RHO_KG_M3,
            gravity_ms2: DEFAULT_G_MS2,
        }
    }
}

impl HydroPlantConfig {
    /// Parse and validate a plant config from a JSON string.
    pub fn from_json(json: &str) -> Result<Self> {
        let config: Self = serde_json::from_str(json)?;
        config.validate()?;
        Ok(config)
    }

    /// Parse from a serde JSON value.
    pub fn from_value(value: serde_json::Value) -> Result<Self> {
        let config: Self = serde_json::from_value(value)?;
        config.validate()?;
        Ok(config)
    }

    /// Basic physical / schema checks.
    pub fn validate(&self) -> Result<()> {
        if self.schema_version == 0 {
            return Err(CoreError::InvalidConfig(
                "schemaVersion must be >= 1".into(),
            ));
        }
        if self.kind != "hydro-plant" {
            return Err(CoreError::InvalidConfig(format!(
                "expected kind \"hydro-plant\", got \"{}\"",
                self.kind
            )));
        }
        if self.id.trim().is_empty() {
            return Err(CoreError::InvalidConfig("id must be non-empty".into()));
        }
        if self.stream.available_flow_m3s < 0.0 {
            return Err(CoreError::InvalidConfig(
                "stream.availableFlowM3s must be >= 0".into(),
            ));
        }
        if self.penstock.gross_head_m < 0.0 {
            return Err(CoreError::InvalidConfig(
                "penstock.grossHeadM must be >= 0".into(),
            ));
        }
        if self.penstock.length_m < 0.0 {
            return Err(CoreError::InvalidConfig(
                "penstock.lengthM must be >= 0".into(),
            ));
        }
        if self.penstock.diameter_m <= 0.0 {
            return Err(CoreError::InvalidConfig(
                "penstock.diameterM must be > 0".into(),
            ));
        }
        if self.penstock.friction_factor < 0.0 {
            return Err(CoreError::InvalidConfig(
                "penstock.frictionFactor must be >= 0".into(),
            ));
        }
        if self.penstock.minor_loss_coefficient < 0.0 {
            return Err(CoreError::InvalidConfig(
                "penstock.minorLossCoefficient must be >= 0".into(),
            ));
        }
        if !(0.0..=1.0).contains(&self.turbine.efficiency) {
            return Err(CoreError::InvalidConfig(
                "turbine.efficiency must be in [0, 1]".into(),
            ));
        }
        if self.turbine.design_flow_m3s < 0.0 {
            return Err(CoreError::InvalidConfig(
                "turbine.designFlowM3s must be >= 0".into(),
            ));
        }
        if let Some(max) = self.turbine.max_safe_flow_m3s {
            if max < 0.0 {
                return Err(CoreError::InvalidConfig(
                    "turbine.maxSafeFlowM3s must be >= 0".into(),
                ));
            }
        }
        if self.turbine.design_speed_rpm < 0.0 {
            return Err(CoreError::InvalidConfig(
                "turbine.designSpeedRpm must be >= 0".into(),
            ));
        }
        for (name, v) in [
            ("speedRampUpS", self.turbine.dynamics.speed_ramp_up_s),
            ("speedRampDownS", self.turbine.dynamics.speed_ramp_down_s),
            ("powerRampUpS", self.turbine.dynamics.power_ramp_up_s),
            ("powerRampDownS", self.turbine.dynamics.power_ramp_down_s),
        ] {
            if v < 0.0 {
                return Err(CoreError::InvalidConfig(format!(
                    "turbine.dynamics.{name} must be >= 0"
                )));
            }
        }
        if !(0.0..=1.0).contains(&self.generator.efficiency) {
            return Err(CoreError::InvalidConfig(
                "generator.efficiency must be in [0, 1]".into(),
            ));
        }
        if self.generator.rated_power_kw < 0.0 {
            return Err(CoreError::InvalidConfig(
                "generator.ratedPowerKw must be >= 0".into(),
            ));
        }
        if self.fluid.density_kg_m3 <= 0.0 {
            return Err(CoreError::InvalidConfig(
                "fluid.densityKgM3 must be > 0".into(),
            ));
        }
        if self.fluid.gravity_ms2 <= 0.0 {
            return Err(CoreError::InvalidConfig(
                "fluid.gravityMs2 must be > 0".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "schemaVersion": 1,
      "kind": "hydro-plant",
      "id": "clearwater-diversion",
      "label": "Clearwater Diversion (Clearwater Run)",
      "stream": { "availableFlowM3s": 0.05 },
      "penstock": {
        "grossHeadM": 25.0,
        "lengthM": 180.0,
        "diameterM": 0.25,
        "frictionFactor": 0.02,
        "minorLossCoefficient": 0.5
      },
      "turbine": {
        "efficiency": 0.75,
        "designFlowM3s": 0.04,
        "maxSafeFlowM3s": 0.06,
        "packageId": "acme-micro-pelton-3kw"
      },
      "generator": {
        "efficiency": 0.92,
        "ratedPowerKw": 8.0
      },
      "fluid": {
        "densityKgM3": 1000.0,
        "gravityMs2": 9.80665
      }
    }"#;

    #[test]
    fn parse_sample_allows_package_id() {
        let cfg = HydroPlantConfig::from_json(SAMPLE).expect("valid sample");
        assert_eq!(cfg.id, "clearwater-diversion");
        assert_eq!(
            cfg.turbine.package_id.as_deref(),
            Some("acme-micro-pelton-3kw")
        );
        assert!((cfg.fluid.density_kg_m3 - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_bad_kind() {
        let bad = SAMPLE.replace("hydro-plant", "solar-plant");
        let err = HydroPlantConfig::from_json(&bad).unwrap_err();
        assert!(err.to_string().contains("kind"));
    }

    #[test]
    fn fluid_defaults_when_omitted() {
        let json = r#"{
          "schemaVersion": 1,
          "kind": "hydro-plant",
          "id": "tiny",
          "stream": { "availableFlowM3s": 0.01 },
          "penstock": { "grossHeadM": 10.0, "lengthM": 50.0, "diameterM": 0.15 },
          "turbine": { "efficiency": 0.8, "designFlowM3s": 0.01 },
          "generator": { "efficiency": 0.9, "ratedPowerKw": 1.0 }
        }"#;
        let cfg = HydroPlantConfig::from_json(json).unwrap();
        assert!((cfg.fluid.gravity_ms2 - DEFAULT_G_MS2).abs() < 1e-12);
        assert!((cfg.penstock.friction_factor - 0.02).abs() < 1e-12);
    }
}
