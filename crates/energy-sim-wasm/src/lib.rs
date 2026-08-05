//! WASM bindings for in-browser hosts (Atomic Adventures alpha, teaching pages).
//!
//! Build with:
//! ```sh
//! wasm-pack build crates/energy-sim-wasm --target web
//! ```
//!
//! Short-term game deploy runs on the player device via this crate. Prefer the
//! long-lived [`WasmSession`] handle for control-room / holo work. One-shot
//! helpers remain for simple teaching pages. The remote REST + WebSocket service
//! remains the designer lab path and the long-term hosted path.

use energy_sim_core::{evaluate_plant_with_inputs, HydroPlantConfig, OperatorInputs};
use energy_sim_runtime::{Command, Session};
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Library version string.
#[wasm_bindgen]
pub fn version() -> String {
    format!(
        "energy-sim-wasm {} (core {}, runtime {})",
        env!("CARGO_PKG_VERSION"),
        energy_sim_core::VERSION,
        energy_sim_runtime::VERSION
    )
}

/// Install better panic messages in the browser console.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Evaluate a hydro plant JSON string with optional operator overrides.
///
/// `operator_json` may be `null`/omitted for defaults (gate open, online).
/// Returns a JS object matching `HydroEvaluation` camelCase fields.
#[wasm_bindgen(js_name = evaluateHydro)]
pub fn evaluate_hydro(
    plant_json: &str,
    operator_json: Option<String>,
) -> Result<JsValue, JsValue> {
    let plant = HydroPlantConfig::from_json(plant_json).map_err(to_js_err)?;
    let inputs = if let Some(op) = operator_json {
        serde_json::from_str::<OperatorInputs>(&op).map_err(to_js_err)?
    } else {
        OperatorInputs::default()
    };
    let eval = evaluate_plant_with_inputs(&plant, &inputs).map_err(to_js_err)?;
    to_js(&eval)
}

/// Create a session from plant or station JSON, start it, advance, return report.
///
/// Prefer [`WasmSession`] for multi-step control-room or holo trials.
#[wasm_bindgen(js_name = runSession)]
pub fn run_session(
    config_json: &str,
    duration_secs: f64,
    commands_json: Option<String>,
) -> Result<JsValue, JsValue> {
    let mut session = Session::from_json(config_json).map_err(to_js_err)?;
    session.start().map_err(to_js_err)?;
    if let Some(cmds) = commands_json {
        apply_commands_json(&mut session, &cmds)?;
    }
    let report = session.advance_secs(duration_secs).map_err(to_js_err)?;
    to_js(&report)
}

/// One-shot session status helper: configure + optional start without advance.
#[wasm_bindgen(js_name = sessionSnapshot)]
pub fn session_snapshot(config_json: &str, start: bool) -> Result<JsValue, JsValue> {
    let mut session = Session::from_json(config_json).map_err(to_js_err)?;
    if start {
        session.start().map_err(to_js_err)?;
    }
    to_js(&session.snapshot())
}

/// Long-lived simulation session for browser hosts (game control room, holo trials).
///
/// Create from plant or station JSON, then call `start`, `advance` / `tick`,
/// `applyCommands`, and `snapshot` repeatedly. Dropping the handle frees WASM
/// memory (or call `.free()` from JS).
///
/// Mirrors the HTTP session contract in `energy-sim-server` so hosts can swap
/// transports via a thin adapter.
#[wasm_bindgen(js_name = Session)]
pub struct WasmSession {
    inner: Session,
}

#[wasm_bindgen(js_class = Session)]
impl WasmSession {
    /// Create a session from plant or full station JSON (not yet started).
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Result<WasmSession, JsValue> {
        let inner = Session::from_json(config_json).map_err(to_js_err)?;
        Ok(WasmSession { inner })
    }

    /// Restore a session from a checkpoint document JSON string.
    #[wasm_bindgen(js_name = fromCheckpoint)]
    pub fn from_checkpoint(checkpoint_json: &str) -> Result<WasmSession, JsValue> {
        let inner = Session::from_checkpoint_json(checkpoint_json).map_err(to_js_err)?;
        Ok(WasmSession { inner })
    }

    /// Start the session; returns the current snapshot.
    pub fn start(&mut self) -> Result<JsValue, JsValue> {
        self.inner.start().map_err(to_js_err)?;
        to_js(&self.inner.snapshot())
    }

    /// Stop the session (gate closed, offline ramp targets); returns snapshot.
    pub fn stop(&mut self) -> Result<JsValue, JsValue> {
        self.inner.stop().map_err(to_js_err)?;
        to_js(&self.inner.snapshot())
    }

    /// Advance simulation time by `duration_secs`.
    ///
    /// Optional `commands_json` is a JSON array of `Command` objects applied
    /// before the advance (same shape as HTTP `POST .../advance`).
    pub fn advance(
        &mut self,
        duration_secs: f64,
        commands_json: Option<String>,
    ) -> Result<JsValue, JsValue> {
        if let Some(cmds) = commands_json {
            apply_commands_json(&mut self.inner, &cmds)?;
        }
        let report = self.inner.advance_secs(duration_secs).map_err(to_js_err)?;
        to_js(&report)
    }

    /// Single live tick of `dt_secs`; returns snapshot.
    pub fn tick(&mut self, dt_secs: f64) -> Result<JsValue, JsValue> {
        let snap = self.inner.tick_secs(dt_secs).map_err(to_js_err)?;
        to_js(&snap)
    }

    /// Apply a JSON array of commands; returns snapshot.
    #[wasm_bindgen(js_name = applyCommands)]
    pub fn apply_commands(&mut self, commands_json: &str) -> Result<JsValue, JsValue> {
        apply_commands_json(&mut self.inner, commands_json)?;
        to_js(&self.inner.snapshot())
    }

    /// Point-in-time snapshot (any phase).
    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        to_js(&self.inner.snapshot())
    }

    /// Events + samples, optionally filtered by sim-time range.
    ///
    /// Returns `{ events, samples }` with camelCase fields.
    pub fn history(
        &self,
        from_secs: Option<f64>,
        to_secs: Option<f64>,
    ) -> Result<JsValue, JsValue> {
        let (events, samples) = self.inner.history_window(from_secs, to_secs);
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct History<'a> {
            events: &'a [energy_sim_runtime::Event],
            samples: &'a [energy_sim_runtime::Sample],
        }
        to_js(&History {
            events: &events,
            samples: &samples,
        })
    }

    /// Full checkpoint document as a JS object (for save / transfer).
    pub fn checkpoint(&self) -> Result<JsValue, JsValue> {
        let value = self.inner.checkpoint_value().map_err(to_js_err)?;
        to_js(&value)
    }

    /// Checkpoint as a JSON string (handy for localStorage).
    #[wasm_bindgen(js_name = checkpointJson)]
    pub fn checkpoint_json(&self) -> Result<String, JsValue> {
        let value = self.inner.checkpoint_value().map_err(to_js_err)?;
        serde_json::to_string(&value).map_err(to_js_err)
    }

    /// Current phase string: `configured` | `running` | `stopped`.
    pub fn phase(&self) -> String {
        match self.inner.phase() {
            energy_sim_runtime::SessionPhase::Configured => "configured".into(),
            energy_sim_runtime::SessionPhase::Running => "running".into(),
            energy_sim_runtime::SessionPhase::Stopped => "stopped".into(),
        }
    }

    /// Current simulation time in seconds.
    #[wasm_bindgen(js_name = simTimeS)]
    pub fn sim_time_s(&self) -> f64 {
        self.inner.sim_time_s()
    }
}

fn apply_commands_json(session: &mut Session, commands_json: &str) -> Result<(), JsValue> {
    let commands: Vec<Command> = serde_json::from_str(commands_json).map_err(to_js_err)?;
    for cmd in commands {
        session.apply(cmd).map_err(to_js_err)?;
    }
    Ok(())
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(to_js_err)
}

fn to_js_err(err: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use energy_sim_runtime::Command;

    fn station_json() -> &'static str {
        include_str!("../../../fixtures/stations/clearwater-station.json")
    }

    fn plant_json() -> &'static str {
        include_str!("../../../fixtures/plants/ideal-teaching.json")
    }

    #[test]
    fn version_nonempty() {
        assert!(version().contains("energy-sim-wasm"));
    }

    #[test]
    fn evaluate_ideal_plant() {
        let plant = HydroPlantConfig::from_json(plant_json()).unwrap();
        let eval = evaluate_plant_with_inputs(&plant, &OperatorInputs::default()).unwrap();
        assert!(eval.electrical_power_kw > 1.0);
    }

    /// Long-lived session path: spin-up, toggle loads, brownout, checkpoint restore.
    ///
    /// Mirrors the WasmSession method sequence (create → start → advance → commands
    /// → history → checkpoint) using the same runtime `Session` the WASM handle wraps.
    #[test]
    fn long_lived_session_spin_up_load_brownout_checkpoint() {
        let mut session = Session::from_json(station_json()).expect("station fixture");
        session.set_sample_period_s(1.0);
        session.start().unwrap();

        session
            .apply(Command::SetHydroInput {
                gate_opening: Some(1.0),
                debris_clog_fraction: None,
                leakage_fraction: None,
                online: Some(true),
            })
            .unwrap();

        let mid = session.advance_secs(30.0).unwrap();
        assert!(
            mid.snapshot.electrical_power_kw > 0.5,
            "expected spin-up power, got {}",
            mid.snapshot.electrical_power_kw
        );
        assert!(mid.snapshot.sim_time_s >= 29.0);
        assert!(
            mid.snapshot.grid_status == "surplus" || mid.snapshot.margin_kw > 0.0,
            "idle loads should leave margin: status={}",
            mid.snapshot.grid_status
        );

        // Campus loads + low gate → generation cannot cover demand.
        for id in ["lighting.main", "ev-charge.port-1", "kitchen.appliance"] {
            session
                .apply(Command::SetLoad {
                    id: id.into(),
                    drawing: true,
                })
                .unwrap();
        }
        session
            .apply(Command::SetHydroInput {
                gate_opening: Some(0.05),
                debris_clog_fraction: None,
                leakage_fraction: None,
                online: None,
            })
            .unwrap();
        // Default power ramp-down is ~25 s; advance past it for a settled deficit.
        let after = session.advance_secs(30.0).unwrap();
        assert!(
            after.snapshot.total_load_kw >= 5.0,
            "expected multi-load draw, got {}",
            after.snapshot.total_load_kw
        );
        assert!(
            after.snapshot.margin_kw < 0.0
                || after.snapshot.grid_status == "brownout"
                || after.snapshot.grid_status == "shortage",
            "expected deficit under loads+low gate: margin={} status={} gen={}",
            after.snapshot.margin_kw,
            after.snapshot.grid_status,
            after.snapshot.electrical_power_kw
        );

        let ck = session.checkpoint_value().unwrap();
        let restored = Session::from_checkpoint_value(ck).unwrap();
        assert!((restored.sim_time_s() - session.sim_time_s()).abs() < 1e-9);
        assert_eq!(
            restored.snapshot().grid_status,
            session.snapshot().grid_status
        );
        assert_eq!(
            restored.snapshot().total_load_kw,
            session.snapshot().total_load_kw
        );

        let (events, samples) = restored.history_window(Some(0.0), None);
        assert!(!events.is_empty());
        assert!(!samples.is_empty());
    }
}
