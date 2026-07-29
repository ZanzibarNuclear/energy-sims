//! Events and time-series samples.

use serde::{Deserialize, Serialize};

/// Discrete operational event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub sim_time_s: f64,
    pub kind: EventKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    ConfigLoaded,
    Started,
    Stopped,
    Command,
    Tick,
    IntervalComplete,
    Checkpoint,
    Warning,
    GridStatusChanged,
    BrownoutEntered,
    BrownoutCleared,
}

/// Continuous sample for charts / CSV export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    pub sim_time_s: f64,
    pub electrical_power_kw: f64,
    pub target_electrical_power_kw: f64,
    pub turbine_speed_rpm: f64,
    pub target_turbine_speed_rpm: f64,
    pub flow_m3s: f64,
    pub net_head_m: f64,
    pub available_generation_kw: f64,
    pub total_load_kw: f64,
    pub margin_kw: f64,
    pub grid_status: String,
    pub energy_generated_kwh: f64,
}
