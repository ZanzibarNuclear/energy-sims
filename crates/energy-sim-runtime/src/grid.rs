//! Station bus, loads, and report-only brownout / shortage balance.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::{Result, RuntimeError};

/// Grid balance status reported to hosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridStatus {
    Ok,
    Surplus,
    Shortage,
    Brownout,
}

impl GridStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Surplus => "surplus",
            Self::Shortage => "shortage",
            Self::Brownout => "brownout",
        }
    }
}

impl std::fmt::Display for GridStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Priority hint for future shed policies (Stage 1 is report-only).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LoadPriority {
    Critical,
    #[default]
    Normal,
    Deferrable,
}

/// Single named load on the station bus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadConfig {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Nameplate demand when drawing (watts).
    pub rating_w: f64,
    #[serde(default)]
    pub priority: LoadPriority,
    /// Initial drawing state when the registry is loaded.
    #[serde(default)]
    pub initially_drawing: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
}

/// Brownout reporting policy (Stage 1: report-only).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrownoutConfig {
    /// Deficit (W) at or above which status becomes `brownout` rather than `shortage`.
    #[serde(default = "default_deficit_threshold_w")]
    pub deficit_threshold_w: f64,
    /// Always `report-only` in Stage 1; other values are accepted but not acted on.
    #[serde(default = "default_brownout_policy")]
    pub policy: String,
}

fn default_deficit_threshold_w() -> f64 {
    1.0
}

fn default_brownout_policy() -> String {
    "report-only".into()
}

impl Default for BrownoutConfig {
    fn default() -> Self {
        Self {
            deficit_threshold_w: default_deficit_threshold_w(),
            policy: default_brownout_policy(),
        }
    }
}

/// Station grid configuration document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationGridConfig {
    pub schema_version: u32,
    pub kind: String,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub loads: Vec<LoadConfig>,
    #[serde(default)]
    pub brownout: BrownoutConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package_id: Option<String>,
}

impl StationGridConfig {
    pub fn from_json(json: &str) -> Result<Self> {
        let cfg: Self = serde_json::from_str(json)?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn from_value(value: serde_json::Value) -> Result<Self> {
        let cfg: Self = serde_json::from_value(value)?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version == 0 {
            return Err(RuntimeError::InvalidConfig(
                "grid schemaVersion must be >= 1".into(),
            ));
        }
        if self.kind != "station-grid" {
            return Err(RuntimeError::InvalidConfig(format!(
                "expected grid kind \"station-grid\", got \"{}\"",
                self.kind
            )));
        }
        if self.id.trim().is_empty() {
            return Err(RuntimeError::InvalidConfig("grid id must be non-empty".into()));
        }
        let mut seen = std::collections::HashSet::new();
        for load in &self.loads {
            if load.id.trim().is_empty() {
                return Err(RuntimeError::InvalidConfig("load id must be non-empty".into()));
            }
            if !seen.insert(load.id.clone()) {
                return Err(RuntimeError::InvalidConfig(format!(
                    "duplicate load id \"{}\"",
                    load.id
                )));
            }
            if load.rating_w < 0.0 {
                return Err(RuntimeError::InvalidConfig(format!(
                    "load {} ratingW must be >= 0",
                    load.id
                )));
            }
        }
        if self.brownout.deficit_threshold_w < 0.0 {
            return Err(RuntimeError::InvalidConfig(
                "brownout.deficitThresholdW must be >= 0".into(),
            ));
        }
        Ok(())
    }
}

/// Live balance result for a tick / snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridBalance {
    pub available_generation_kw: f64,
    pub total_load_kw: f64,
    pub margin_kw: f64,
    pub bus_energized: bool,
    pub status: GridStatus,
}

/// Runtime grid state: config + which loads are drawing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationGrid {
    pub config: StationGridConfig,
    /// Load id → currently drawing.
    pub drawing: BTreeMap<String, bool>,
}

impl StationGrid {
    pub fn new(config: StationGridConfig) -> Self {
        let mut drawing = BTreeMap::new();
        for load in &config.loads {
            drawing.insert(load.id.clone(), load.initially_drawing);
        }
        Self { config, drawing }
    }

    pub fn set_load_drawing(&mut self, id: &str, drawing: bool) -> Result<()> {
        if !self.config.loads.iter().any(|l| l.id == id) {
            return Err(RuntimeError::UnknownLoad(id.to_string()));
        }
        self.drawing.insert(id.to_string(), drawing);
        Ok(())
    }

    /// Merge external drawing map (e.g. from checkpoint) for known loads.
    pub fn apply_drawing_map(&mut self, map: &BTreeMap<String, bool>) {
        for (id, drawing) in map {
            if self.drawing.contains_key(id) {
                self.drawing.insert(id.clone(), *drawing);
            }
        }
    }

    /// Sum of drawing load ratings (kW).
    pub fn total_load_kw(&self) -> f64 {
        let mut watts = 0.0;
        for load in &self.config.loads {
            if self.drawing.get(&load.id).copied().unwrap_or(false) {
                watts += load.rating_w;
            }
        }
        watts / 1000.0
    }

    /// Host-facing load rows (id, label, rating, priority, drawing).
    pub fn load_snapshots(&self) -> Vec<crate::snapshot::LoadSnapshot> {
        self.config
            .loads
            .iter()
            .map(|load| crate::snapshot::LoadSnapshot {
                id: load.id.clone(),
                label: load.label.clone(),
                rating_w: load.rating_w,
                priority: load.priority,
                drawing: self.drawing.get(&load.id).copied().unwrap_or(false),
            })
            .collect()
    }

    /// Balance generation against drawing loads. **Report-only** brownout.
    pub fn balance(&self, available_generation_kw: f64) -> GridBalance {
        let total_load_kw = self.total_load_kw();
        let margin_kw = available_generation_kw - total_load_kw;
        let deficit_w = (-margin_kw).max(0.0) * 1000.0;
        let threshold = self.config.brownout.deficit_threshold_w;

        let status = if total_load_kw <= 0.0 && available_generation_kw <= 0.0 {
            GridStatus::Ok
        } else if margin_kw < 0.0 {
            if deficit_w >= threshold {
                GridStatus::Brownout
            } else {
                GridStatus::Shortage
            }
        } else if margin_kw > 0.0 {
            GridStatus::Surplus
        } else {
            GridStatus::Ok
        };

        // Bus is energized when there is positive generation available (report-only:
        // we still energize under deficit so hosts can present brownout on a live bus).
        let bus_energized = available_generation_kw > 0.0;

        GridBalance {
            available_generation_kw,
            total_load_kw,
            margin_kw,
            bus_energized,
            status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_grid() -> StationGrid {
        let cfg = StationGridConfig::from_json(
            r#"{
              "schemaVersion": 1,
              "kind": "station-grid",
              "id": "clearwater-station",
              "loads": [
                { "id": "lighting.main", "label": "Main lights", "ratingW": 400, "priority": "normal" },
                { "id": "ev-charge.port-1", "label": "EV", "ratingW": 3500, "priority": "deferrable" }
              ],
              "brownout": { "deficitThresholdW": 1, "policy": "report-only" }
            }"#,
        )
        .unwrap();
        StationGrid::new(cfg)
    }

    #[test]
    fn surplus_when_gen_exceeds_load() {
        let mut g = sample_grid();
        g.set_load_drawing("lighting.main", true).unwrap();
        let b = g.balance(2.0); // 2 kW vs 0.4 kW
        assert_eq!(b.status, GridStatus::Surplus);
        assert!(b.margin_kw > 0.0);
        assert!(b.bus_energized);
    }

    #[test]
    fn load_snapshots_reflect_drawing() {
        let mut g = sample_grid();
        g.set_load_drawing("ev-charge.port-1", true).unwrap();
        let rows = g.load_snapshots();
        assert_eq!(rows.len(), 2);
        let lighting = rows.iter().find(|r| r.id == "lighting.main").unwrap();
        assert!(!lighting.drawing);
        let ev = rows.iter().find(|r| r.id == "ev-charge.port-1").unwrap();
        assert!(ev.drawing);
        assert!((ev.rating_w - 3500.0).abs() < 1e-9);
        assert_eq!(ev.priority, LoadPriority::Deferrable);
    }

    #[test]
    fn brownout_when_deficit_above_threshold() {
        let mut g = sample_grid();
        g.set_load_drawing("lighting.main", true).unwrap();
        g.set_load_drawing("ev-charge.port-1", true).unwrap();
        // 3.9 kW load, 1 kW gen → deficit 2.9 kW >> 1 W threshold
        let b = g.balance(1.0);
        assert_eq!(b.status, GridStatus::Brownout);
        assert!(b.margin_kw < 0.0);
        assert!(b.bus_energized); // still report bus live; host dims lights
    }

    #[test]
    fn no_auto_shed_loads_stay_drawing() {
        let mut g = sample_grid();
        g.set_load_drawing("ev-charge.port-1", true).unwrap();
        let _ = g.balance(0.1);
        assert_eq!(g.drawing.get("ev-charge.port-1"), Some(&true));
    }

    #[test]
    fn unknown_load_errors() {
        let mut g = sample_grid();
        assert!(g.set_load_drawing("nope", true).is_err());
    }
}
