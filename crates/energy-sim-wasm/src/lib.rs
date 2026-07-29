//! Optional WASM bindings for embedded hosts (teaching pages, offline demos).
//!
//! Build with:
//! ```sh
//! wasm-pack build crates/energy-sim-wasm --target web
//! ```
//!
//! The primary game path remains the remote REST + WebSocket service.

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

/// Create a session from plant or station JSON, start it, advance, return snapshot.
#[wasm_bindgen(js_name = runSession)]
pub fn run_session(
    config_json: &str,
    duration_secs: f64,
    commands_json: Option<String>,
) -> Result<JsValue, JsValue> {
    let mut session = Session::from_json(config_json).map_err(to_js_err)?;
    session.start().map_err(to_js_err)?;
    if let Some(cmds) = commands_json {
        let commands: Vec<Command> = serde_json::from_str(&cmds).map_err(to_js_err)?;
        for cmd in commands {
            session.apply(cmd).map_err(to_js_err)?;
        }
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

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(to_js_err)
}

fn to_js_err(err: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_nonempty() {
        assert!(version().contains("energy-sim-wasm"));
    }

    #[test]
    fn evaluate_ideal_plant() {
        let json = include_str!("../../../fixtures/plants/ideal-teaching.json");
        let plant = HydroPlantConfig::from_json(json).unwrap();
        let eval = evaluate_plant_with_inputs(&plant, &OperatorInputs::default()).unwrap();
        assert!(eval.electrical_power_kw > 1.0);
    }
}
