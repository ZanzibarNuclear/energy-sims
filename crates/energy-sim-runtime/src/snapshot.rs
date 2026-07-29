//! Point-in-time session snapshot for hosts and CLI status.

use serde::{Deserialize, Serialize};

use crate::session::SessionPhase;

/// Full operational snapshot (actuals + optional targets + grid).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub sim_time_s: f64,
    pub phase: SessionPhase,
    pub plant_id: String,
    pub flow_m3s: f64,
    pub gross_head_m: f64,
    pub net_head_m: f64,
    pub head_loss_m: f64,
    pub hydraulic_power_kw: f64,
    pub electrical_power_kw: f64,
    pub target_electrical_power_kw: f64,
    pub turbine_speed_rpm: f64,
    pub target_turbine_speed_rpm: f64,
    pub energy_generated_kwh: f64,
    pub available_generation_kw: f64,
    pub total_load_kw: f64,
    pub margin_kw: f64,
    pub bus_energized: bool,
    pub grid_status: String,
    pub warnings: Vec<String>,
}
