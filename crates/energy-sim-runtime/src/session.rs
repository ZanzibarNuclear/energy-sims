//! Session lifecycle: configure, start/stop, advance/tick, snapshot, checkpoint.

use std::path::Path;

use energy_sim_core::{
    evaluate_plant_with_inputs, HydroPlantConfig, OperatorInputs, WATT_S_PER_KWH,
};
use serde::{Deserialize, Serialize};

use crate::dynamics::approach;
use crate::error::{Result, RuntimeError};
use crate::export::{write_events_jsonl, write_series_csv};
use crate::grid::{GridStatus, StationGrid, StationGridConfig};
use crate::history::{Event, EventKind, Sample};
use crate::snapshot::Snapshot;

/// Lifecycle phase of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionPhase {
    Configured,
    Running,
    Stopped,
}

/// Composite session configuration (plant required; grid optional).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfig {
    pub schema_version: u32,
    #[serde(default = "default_session_kind")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub plant: HydroPlantConfig,
    /// Optional station grid (loads + brownout policy).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid: Option<StationGridConfig>,
}

fn default_session_kind() -> String {
    "energy-session".into()
}

impl SessionConfig {
    pub fn from_json(json: &str) -> Result<Self> {
        // Accept either a full session document or a bare hydro-plant document.
        let value: serde_json::Value = serde_json::from_str(json)?;
        Self::from_value(value)
    }

    pub fn from_value(value: serde_json::Value) -> Result<Self> {
        let kind = value
            .get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or("hydro-plant");

        if kind == "hydro-plant" {
            let plant = HydroPlantConfig::from_value(value)?;
            return Ok(Self {
                schema_version: plant.schema_version,
                kind: "energy-session".into(),
                id: Some(plant.id.clone()),
                plant,
                grid: None,
            });
        }

        let config: Self = serde_json::from_value(value)?;
        config.plant.validate()?;
        if let Some(ref grid) = config.grid {
            grid.validate()?;
        }
        if config.schema_version == 0 {
            return Err(RuntimeError::InvalidConfig(
                "schemaVersion must be >= 1".into(),
            ));
        }
        Ok(config)
    }
}

/// Mutable hydro operator knobs for the session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HydroOperatorState {
    pub gate_opening: f64,
    pub debris_clog_fraction: f64,
    pub leakage_fraction: f64,
    pub online: bool,
}

impl Default for HydroOperatorState {
    fn default() -> Self {
        Self {
            gate_opening: 1.0,
            debris_clog_fraction: 0.0,
            leakage_fraction: 0.0,
            online: true,
        }
    }
}

impl From<&HydroOperatorState> for OperatorInputs {
    fn from(s: &HydroOperatorState) -> Self {
        Self {
            gate_opening: s.gate_opening,
            debris_clog_fraction: s.debris_clog_fraction,
            leakage_fraction: s.leakage_fraction,
            online: s.online,
        }
    }
}

/// Commands hosts may apply between or during advances.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    SetHydroInput {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        gate_opening: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        debris_clog_fraction: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        leakage_fraction: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        online: Option<bool>,
    },
    /// Load commands are accepted but only take effect once the grid module (PR4) is active.
    SetLoad {
        id: String,
        drawing: bool,
    },
}

/// Result of advancing a known interval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvanceReport {
    pub snapshot: Snapshot,
    pub energy_interval_kwh: f64,
    pub samples_added: usize,
    pub events_added: usize,
}

/// Serializable checkpoint for resume.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckpointDocument {
    schema_version: u32,
    kind: String,
    config: SessionConfig,
    phase: SessionPhase,
    sim_time_s: f64,
    operator: HydroOperatorState,
    actual_power_kw: f64,
    actual_speed_rpm: f64,
    energy_generated_kwh: f64,
    events: Vec<Event>,
    samples: Vec<Sample>,
    /// Load drawing map (id → drawing); used by PR4 grid.
    #[serde(default)]
    load_drawing: std::collections::BTreeMap<String, bool>,
}

/// One running simulation instance.
#[derive(Debug, Clone)]
pub struct Session {
    config: SessionConfig,
    phase: SessionPhase,
    sim_time_s: f64,
    operator: HydroOperatorState,
    actual_power_kw: f64,
    actual_speed_rpm: f64,
    energy_generated_kwh: f64,
    events: Vec<Event>,
    samples: Vec<Sample>,
    load_drawing: std::collections::BTreeMap<String, bool>,
    grid: Option<StationGrid>,
    last_grid_status: Option<GridStatus>,
    /// Sample period while integrating long intervals (seconds).
    sample_period_s: f64,
}

impl Session {
    /// Create a session from JSON (session document or bare plant).
    pub fn from_json(json: &str) -> Result<Self> {
        let config = SessionConfig::from_json(json)?;
        Ok(Self::from_config(config))
    }

    pub fn from_config(config: SessionConfig) -> Self {
        let mut events = Vec::new();
        events.push(Event {
            sim_time_s: 0.0,
            kind: EventKind::ConfigLoaded,
            message: Some(format!("plant {}", config.plant.id)),
            detail: None,
        });
        let grid = config.grid.clone().map(StationGrid::new);
        let mut load_drawing = std::collections::BTreeMap::new();
        if let Some(ref g) = grid {
            load_drawing = g.drawing.clone();
        }
        Self {
            config,
            phase: SessionPhase::Configured,
            sim_time_s: 0.0,
            operator: HydroOperatorState::default(),
            actual_power_kw: 0.0,
            actual_speed_rpm: 0.0,
            energy_generated_kwh: 0.0,
            events,
            samples: Vec::new(),
            load_drawing,
            grid,
            last_grid_status: None,
            sample_period_s: 1.0,
        }
    }

    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    pub fn phase(&self) -> SessionPhase {
        self.phase
    }

    pub fn sim_time_s(&self) -> f64 {
        self.sim_time_s
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn samples(&self) -> &[Sample] {
        &self.samples
    }

    pub fn set_sample_period_s(&mut self, period_s: f64) {
        if period_s > 0.0 {
            self.sample_period_s = period_s;
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.operator.online = true;
        self.phase = SessionPhase::Running;
        self.push_event(EventKind::Started, Some("session started".into()), None);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        // Command targets toward offline; ramps bring actuals down on subsequent ticks.
        self.operator.online = false;
        self.operator.gate_opening = 0.0;
        self.phase = SessionPhase::Stopped;
        self.push_event(EventKind::Stopped, Some("session stopped".into()), None);
        Ok(())
    }

    pub fn apply(&mut self, command: Command) -> Result<()> {
        match command {
            Command::SetHydroInput {
                gate_opening,
                debris_clog_fraction,
                leakage_fraction,
                online,
            } => {
                if let Some(g) = gate_opening {
                    self.operator.gate_opening = g.clamp(0.0, 1.0);
                }
                if let Some(d) = debris_clog_fraction {
                    self.operator.debris_clog_fraction = d.clamp(0.0, 1.0);
                }
                if let Some(l) = leakage_fraction {
                    self.operator.leakage_fraction = l.clamp(0.0, 1.0);
                }
                if let Some(o) = online {
                    self.operator.online = o;
                }
                self.push_event(
                    EventKind::Command,
                    Some("set_hydro_input".into()),
                    Some(serde_json::to_value(&self.operator)?),
                );
            }
            Command::SetLoad { id, drawing } => {
                self.load_drawing.insert(id.clone(), drawing);
                if let Some(ref mut grid) = self.grid {
                    grid.set_load_drawing(&id, drawing)?;
                }
                self.push_event(
                    EventKind::Command,
                    Some(format!("set_load {id} drawing={drawing}")),
                    None,
                );
                self.note_grid_status_change();
            }
        }
        Ok(())
    }

    /// Advance a known interval of `duration_s` seconds, integrating ramps.
    pub fn advance_secs(&mut self, duration_s: f64) -> Result<AdvanceReport> {
        if duration_s < 0.0 {
            return Err(RuntimeError::InvalidTime(
                "duration_s must be >= 0".into(),
            ));
        }
        if self.phase != SessionPhase::Running {
            return Err(RuntimeError::NotRunning(self.phase));
        }

        let energy_before = self.energy_generated_kwh;
        let events_before = self.events.len();
        let samples_before = self.samples.len();

        // Record starting sample.
        self.record_sample();

        let mut remaining = duration_s;
        while remaining > 1e-12 {
            let dt = remaining.min(self.sample_period_s);
            self.step(dt)?;
            self.record_sample();
            remaining -= dt;
        }

        self.push_event(
            EventKind::IntervalComplete,
            Some(format!("advanced {duration_s} s")),
            None,
        );

        Ok(AdvanceReport {
            snapshot: self.snapshot(),
            energy_interval_kwh: self.energy_generated_kwh - energy_before,
            samples_added: self.samples.len() - samples_before,
            events_added: self.events.len() - events_before,
        })
    }

    /// Open-ended single step (live ops / interactive).
    pub fn tick_secs(&mut self, dt_s: f64) -> Result<Snapshot> {
        if dt_s < 0.0 {
            return Err(RuntimeError::InvalidTime("dt_s must be >= 0".into()));
        }
        if self.phase != SessionPhase::Running {
            return Err(RuntimeError::NotRunning(self.phase));
        }
        self.step(dt_s)?;
        self.record_sample();
        self.push_event(
            EventKind::Tick,
            Some(format!("tick {dt_s} s")),
            None,
        );
        Ok(self.snapshot())
    }

    /// Current snapshot (works in any phase).
    pub fn snapshot(&self) -> Snapshot {
        let (flow, gross, net, loss, hyd_kw, target_el, target_speed, warnings) =
            match self.evaluate_target() {
                Ok(t) => (
                    t.flow_m3s,
                    t.gross_head_m,
                    t.net_head_m,
                    t.head_loss_m,
                    t.hydraulic_power_kw,
                    t.electrical_power_kw,
                    t.turbine_speed_rpm,
                    t.warnings,
                ),
                Err(_) => (
                    0.0,
                    self.config.plant.penstock.gross_head_m,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    vec!["evaluation failed".into()],
                ),
            };

        let available = self.actual_power_kw;
        let balance = if let Some(ref grid) = self.grid {
            grid.balance(available)
        } else {
            crate::grid::GridBalance {
                available_generation_kw: available,
                total_load_kw: 0.0,
                margin_kw: available,
                bus_energized: available > 0.0,
                status: if available > 0.0 {
                    GridStatus::Surplus
                } else {
                    GridStatus::Ok
                },
            }
        };

        Snapshot {
            sim_time_s: self.sim_time_s,
            phase: self.phase,
            plant_id: self.config.plant.id.clone(),
            flow_m3s: flow,
            gross_head_m: gross,
            net_head_m: net,
            head_loss_m: loss,
            hydraulic_power_kw: hyd_kw,
            electrical_power_kw: self.actual_power_kw,
            target_electrical_power_kw: target_el,
            turbine_speed_rpm: self.actual_speed_rpm,
            target_turbine_speed_rpm: target_speed,
            energy_generated_kwh: self.energy_generated_kwh,
            available_generation_kw: balance.available_generation_kw,
            total_load_kw: balance.total_load_kw,
            margin_kw: balance.margin_kw,
            bus_energized: balance.bus_energized,
            grid_status: balance.status.as_str().into(),
            warnings,
        }
    }

    pub fn export_series_csv(&self, path: impl AsRef<Path>) -> Result<()> {
        write_series_csv(path, &self.samples)
    }

    pub fn export_events_jsonl(&self, path: impl AsRef<Path>) -> Result<()> {
        write_events_jsonl(path, &self.events)
    }

    pub fn save_checkpoint(&self, path: impl AsRef<Path>) -> Result<()> {
        let load_drawing = self
            .grid
            .as_ref()
            .map(|g| g.drawing.clone())
            .unwrap_or_else(|| self.load_drawing.clone());
        let doc = CheckpointDocument {
            schema_version: 1,
            kind: "energy-sim-checkpoint".into(),
            config: self.config.clone(),
            phase: self.phase,
            sim_time_s: self.sim_time_s,
            operator: self.operator.clone(),
            actual_power_kw: self.actual_power_kw,
            actual_speed_rpm: self.actual_speed_rpm,
            energy_generated_kwh: self.energy_generated_kwh,
            events: self.events.clone(),
            samples: self.samples.clone(),
            load_drawing,
        };
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, &doc)?;
        Ok(())
    }

    pub fn load_checkpoint(path: impl AsRef<Path>) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let doc: CheckpointDocument = serde_json::from_reader(file)?;
        doc.config.plant.validate()?;
        if let Some(ref g) = doc.config.grid {
            g.validate()?;
        }
        let mut grid = doc.config.grid.clone().map(StationGrid::new);
        if let Some(ref mut g) = grid {
            g.apply_drawing_map(&doc.load_drawing);
        }
        Ok(Self {
            config: doc.config,
            phase: doc.phase,
            sim_time_s: doc.sim_time_s,
            operator: doc.operator,
            actual_power_kw: doc.actual_power_kw,
            actual_speed_rpm: doc.actual_speed_rpm,
            energy_generated_kwh: doc.energy_generated_kwh,
            events: doc.events,
            samples: doc.samples,
            load_drawing: doc.load_drawing,
            grid,
            last_grid_status: None,
            sample_period_s: 1.0,
        })
    }

    pub fn grid(&self) -> Option<&StationGrid> {
        self.grid.as_ref()
    }

    // --- internals ---

    fn evaluate_target(&self) -> Result<energy_sim_core::HydroEvaluation> {
        let inputs = OperatorInputs::from(&self.operator);
        Ok(evaluate_plant_with_inputs(&self.config.plant, &inputs)?)
    }

    fn step(&mut self, dt_s: f64) -> Result<()> {
        if dt_s == 0.0 {
            return Ok(());
        }
        let target = self.evaluate_target()?;
        let dyn_cfg = &self.config.plant.turbine.dynamics;

        let power_before = self.actual_power_kw;
        self.actual_power_kw = approach(
            self.actual_power_kw,
            target.electrical_power_kw,
            dt_s,
            dyn_cfg.power_ramp_up_s,
            dyn_cfg.power_ramp_down_s,
        );
        self.actual_speed_rpm = approach(
            self.actual_speed_rpm,
            target.turbine_speed_rpm,
            dt_s,
            dyn_cfg.speed_ramp_up_s,
            dyn_cfg.speed_ramp_down_s,
        );

        // Trapezoidal energy integral of actual power (kW · s → kWh).
        let avg_kw = 0.5 * (power_before + self.actual_power_kw);
        let joules = avg_kw * 1000.0 * dt_s;
        self.energy_generated_kwh += joules / WATT_S_PER_KWH;

        self.sim_time_s += dt_s;
        self.note_grid_status_change();
        Ok(())
    }

    fn record_sample(&mut self) {
        let snap = self.snapshot();
        self.samples.push(Sample {
            sim_time_s: snap.sim_time_s,
            electrical_power_kw: snap.electrical_power_kw,
            target_electrical_power_kw: snap.target_electrical_power_kw,
            turbine_speed_rpm: snap.turbine_speed_rpm,
            target_turbine_speed_rpm: snap.target_turbine_speed_rpm,
            flow_m3s: snap.flow_m3s,
            net_head_m: snap.net_head_m,
            available_generation_kw: snap.available_generation_kw,
            total_load_kw: snap.total_load_kw,
            margin_kw: snap.margin_kw,
            grid_status: snap.grid_status.clone(),
            energy_generated_kwh: snap.energy_generated_kwh,
        });
    }

    fn push_event(
        &mut self,
        kind: EventKind,
        message: Option<String>,
        detail: Option<serde_json::Value>,
    ) {
        self.events.push(Event {
            sim_time_s: self.sim_time_s,
            kind,
            message,
            detail,
        });
    }

    fn note_grid_status_change(&mut self) {
        let status = if let Some(ref grid) = self.grid {
            grid.balance(self.actual_power_kw).status
        } else {
            return;
        };
        if self.last_grid_status == Some(status) {
            return;
        }
        let prev = self.last_grid_status;
        self.last_grid_status = Some(status);
        if prev.is_none() {
            return;
        }
        self.push_event(
            EventKind::GridStatusChanged,
            Some(format!("grid status → {status}")),
            None,
        );
        if status == GridStatus::Brownout {
            self.push_event(
                EventKind::BrownoutEntered,
                Some("demand exceeds supply (report-only)".into()),
                None,
            );
        } else if prev == Some(GridStatus::Brownout) {
            self.push_event(
                EventKind::BrownoutCleared,
                Some("brownout cleared".into()),
                None,
            );
        }
    }

    /// Access load drawing map.
    pub fn load_drawing(&self) -> &std::collections::BTreeMap<String, bool> {
        &self.load_drawing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plant_json() -> String {
        std::fs::read_to_string("fixtures/plants/upper-penstock.json").unwrap_or_else(|_| {
            // When tests run from crate dir, path differs.
            std::fs::read_to_string("../../fixtures/plants/upper-penstock.json")
                .expect("fixture path")
        })
    }

    #[test]
    fn spin_up_curve_over_ramp() {
        let mut session = Session::from_json(&plant_json()).unwrap();
        session.set_sample_period_s(5.0);
        session.start().unwrap();
        let report = session.advance_secs(30.0).unwrap();
        assert!(report.samples_added >= 5);
        let samples = session.samples();
        // Power should rise over time toward target (not jump to full on first sample after t=0).
        let first_power = samples
            .iter()
            .find(|s| s.sim_time_s > 0.0)
            .map(|s| s.electrical_power_kw)
            .unwrap();
        let last = samples.last().unwrap();
        assert!(
            last.electrical_power_kw > first_power,
            "expected ramp-up: first={first_power} last={}",
            last.electrical_power_kw
        );
        assert!(
            (last.electrical_power_kw - last.target_electrical_power_kw).abs() < 0.05
                || last.electrical_power_kw <= last.target_electrical_power_kw + 0.05,
            "should approach target"
        );
    }

    #[test]
    fn spin_down_on_stop() {
        let mut session = Session::from_json(&plant_json()).unwrap();
        session.set_sample_period_s(5.0);
        session.start().unwrap();
        session.advance_secs(30.0).unwrap();
        let power_at_full = session.snapshot().electrical_power_kw;
        assert!(power_at_full > 1.0);

        // stop() sets phase Stopped which blocks advance — use apply offline + keep running
        // for spin-down test: design says stop commands targets offline; allow tick while stopped
        // by re-opening running with gate closed.
        session.phase = SessionPhase::Running;
        session
            .apply(Command::SetHydroInput {
                gate_opening: Some(0.0),
                debris_clog_fraction: None,
                leakage_fraction: None,
                online: Some(true),
            })
            .unwrap();
        session.advance_secs(120.0).unwrap();
        let after = session.snapshot().electrical_power_kw;
        assert!(
            after < power_at_full * 0.15,
            "expected spin-down: was {power_at_full}, now {after}"
        );
    }

    #[test]
    fn energy_uses_actual_not_instant_target() {
        let mut session = Session::from_json(&plant_json()).unwrap();
        session.set_sample_period_s(1.0);
        session.start().unwrap();
        let report = session.advance_secs(10.0).unwrap();
        let target = session.snapshot().target_electrical_power_kw;
        // If power jumped instantly, energy ≈ target * 10/3600 kWh.
        let instant_energy = target * 10.0 / 3600.0;
        assert!(
            report.energy_interval_kwh < instant_energy * 0.95,
            "ramped energy {} should be less than instant {}",
            report.energy_interval_kwh,
            instant_energy
        );
    }

    #[test]
    fn checkpoint_round_trip() {
        let dir = std::env::temp_dir().join(format!("energy-sim-ckpt-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("checkpoint.json");

        let mut session = Session::from_json(&plant_json()).unwrap();
        session.start().unwrap();
        session.advance_secs(15.0).unwrap();
        let t = session.sim_time_s();
        let e = session.snapshot().energy_generated_kwh;
        session.save_checkpoint(&path).unwrap();

        let loaded = Session::load_checkpoint(&path).unwrap();
        assert!((loaded.sim_time_s() - t).abs() < 1e-9);
        assert!((loaded.snapshot().energy_generated_kwh - e).abs() < 1e-12);
        assert_eq!(loaded.phase(), SessionPhase::Running);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cannot_advance_before_start() {
        let mut session = Session::from_json(&plant_json()).unwrap();
        let err = session.advance_secs(1.0).unwrap_err();
        assert!(matches!(err, RuntimeError::NotRunning(_)));
    }

    fn station_json() -> String {
        std::fs::read_to_string("fixtures/stations/utility-station.json").unwrap_or_else(|_| {
            std::fs::read_to_string("../../fixtures/stations/utility-station.json")
                .expect("station fixture")
        })
    }

    #[test]
    fn grid_reports_brownout_under_heavy_load() {
        let mut session = Session::from_json(&station_json()).unwrap();
        session.start().unwrap();
        // Snap ramps for a clear balance test.
        session.config.plant.turbine.dynamics.power_ramp_up_s = 0.0;
        session.config.plant.turbine.dynamics.speed_ramp_up_s = 0.0;
        session.advance_secs(1.0).unwrap();

        session
            .apply(Command::SetLoad {
                id: "lighting.main".into(),
                drawing: true,
            })
            .unwrap();
        session
            .apply(Command::SetLoad {
                id: "ev-charge.port-1".into(),
                drawing: true,
            })
            .unwrap();
        session
            .apply(Command::SetLoad {
                id: "kitchen.appliance".into(),
                drawing: true,
            })
            .unwrap();

        // Crush generation so load >> supply.
        session
            .apply(Command::SetHydroInput {
                gate_opening: Some(0.05),
                debris_clog_fraction: None,
                leakage_fraction: None,
                online: None,
            })
            .unwrap();
        session.config.plant.turbine.dynamics.power_ramp_down_s = 0.0;
        session.advance_secs(1.0).unwrap();

        let snap = session.snapshot();
        assert!(
            snap.grid_status == "brownout" || snap.grid_status == "shortage",
            "status={}",
            snap.grid_status
        );
        assert!(snap.margin_kw < 0.0);
        // Report-only: loads still marked drawing.
        assert_eq!(
            session.grid().unwrap().drawing.get("ev-charge.port-1"),
            Some(&true)
        );
    }
}
