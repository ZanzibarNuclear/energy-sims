//! Hydro plant configuration and steady-state power evaluation.

mod config;
mod evaluate;
mod losses;

pub use config::{
    FluidConfig, GeneratorConfig, HydroPlantConfig, PenstockConfig, StreamConfig, TurbineConfig,
};
pub use evaluate::{evaluate_plant, evaluate_plant_with_inputs, HydroEvaluation, OperatorInputs};
pub use losses::{compute_head_loss_m, LossBreakdown};
