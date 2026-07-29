//! Headless operator for the energy simulation engine.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use energy_sim_core::{evaluate_plant, evaluate_plant_with_inputs, HydroPlantConfig, OperatorInputs};
use energy_sim_runtime::{Command, RunPackage, Session};

#[derive(Parser, Debug)]
#[command(
    name = "energy-sim",
    version,
    about = "Headless energy simulation engine (hydro + station grid)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Steady-state hydro plant evaluation (no session clock).
    Hydro {
        #[command(subcommand)]
        command: HydroCommands,
    },
    /// Session lifecycle: run, resume, export, status.
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
    /// Print engine version banner.
    Version,
}

#[derive(Subcommand, Debug)]
enum HydroCommands {
    /// Evaluate plant JSON and print power / head breakdown as JSON.
    Eval {
        /// Path to hydro plant JSON config.
        #[arg(long, short = 'c')]
        config: PathBuf,
        /// Gate opening 0–1 (default 1).
        #[arg(long, default_value_t = 1.0)]
        gate: f64,
        /// Debris clog fraction 0–1.
        #[arg(long, default_value_t = 0.0)]
        debris: f64,
        /// Leakage fraction 0–1.
        #[arg(long, default_value_t = 0.0)]
        leakage: f64,
        /// Mark plant offline (zero delivery).
        #[arg(long, default_value_t = false)]
        offline: bool,
    },
}

#[derive(Subcommand, Debug)]
enum SessionCommands {
    /// Create a session, run for a duration, write outputs.
    Run {
        /// Path to session or plant JSON config.
        #[arg(long, short = 'c')]
        config: PathBuf,
        /// Simulation duration in seconds.
        #[arg(long, default_value_t = 60.0)]
        duration_secs: f64,
        /// Output directory for checkpoint.json, events.jsonl, series.csv.
        #[arg(long, short = 'o')]
        out_dir: PathBuf,
        /// Sample period while integrating (seconds).
        #[arg(long, default_value_t = 1.0)]
        sample_period_secs: f64,
        /// Optional JSON array of commands to apply after start, before advance.
        #[arg(long)]
        commands: Option<PathBuf>,
    },
    /// Resume from checkpoint and advance further.
    Resume {
        #[arg(long)]
        checkpoint: PathBuf,
        #[arg(long, default_value_t = 60.0)]
        duration_secs: f64,
        /// Directory for updated outputs (defaults to checkpoint's parent).
        #[arg(long, short = 'o')]
        out_dir: Option<PathBuf>,
        #[arg(long, default_value_t = 1.0)]
        sample_period_secs: f64,
        #[arg(long)]
        commands: Option<PathBuf>,
    },
    /// Export series or events from a checkpoint.
    Export {
        #[arg(long)]
        checkpoint: PathBuf,
        #[arg(long, value_enum, default_value_t = ExportFormat::Csv)]
        format: ExportFormat,
        #[arg(long, short = 'o')]
        out: PathBuf,
    },
    /// Print current snapshot JSON from a checkpoint.
    Status {
        #[arg(long)]
        checkpoint: PathBuf,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ExportFormat {
    Csv,
    Jsonl,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Version => {
            println!("{}", energy_sim_runtime::engine_banner());
        }
        Commands::Hydro {
            command: HydroCommands::Eval {
                config,
                gate,
                debris,
                leakage,
                offline,
            },
        } => {
            hydro_eval(&config, gate, debris, leakage, offline)?;
        }
        Commands::Session { command } => match command {
            SessionCommands::Run {
                config,
                duration_secs,
                out_dir,
                sample_period_secs,
                commands,
            } => {
                session_run(
                    &config,
                    duration_secs,
                    &out_dir,
                    sample_period_secs,
                    commands.as_deref(),
                )?;
            }
            SessionCommands::Resume {
                checkpoint,
                duration_secs,
                out_dir,
                sample_period_secs,
                commands,
            } => {
                session_resume(
                    &checkpoint,
                    duration_secs,
                    out_dir.as_deref(),
                    sample_period_secs,
                    commands.as_deref(),
                )?;
            }
            SessionCommands::Export {
                checkpoint,
                format,
                out,
            } => {
                session_export(&checkpoint, format, &out)?;
            }
            SessionCommands::Status { checkpoint } => {
                session_status(&checkpoint)?;
            }
        },
    }
    Ok(())
}

fn hydro_eval(
    config: &Path,
    gate: f64,
    debris: f64,
    leakage: f64,
    offline: bool,
) -> Result<()> {
    let text = fs::read_to_string(config)
        .with_context(|| format!("reading config {}", config.display()))?;
    let plant = HydroPlantConfig::from_json(&text)?;
    let inputs = OperatorInputs {
        gate_opening: gate,
        debris_clog_fraction: debris,
        leakage_fraction: leakage,
        online: !offline,
    };
    let eval = if gate == 1.0 && debris == 0.0 && leakage == 0.0 && !offline {
        evaluate_plant(&plant)?
    } else {
        evaluate_plant_with_inputs(&plant, &inputs)?
    };
    println!("{}", serde_json::to_string_pretty(&eval)?);
    Ok(())
}

fn session_run(
    config: &Path,
    duration_secs: f64,
    out_dir: &Path,
    sample_period_secs: f64,
    commands_path: Option<&Path>,
) -> Result<()> {
    if duration_secs < 0.0 {
        bail!("duration-secs must be >= 0");
    }
    let text = fs::read_to_string(config)
        .with_context(|| format!("reading config {}", config.display()))?;
    let mut session = Session::from_json(&text)?;
    session.set_sample_period_s(sample_period_secs);
    session.start()?;
    apply_commands_file(&mut session, commands_path)?;
    let report = session.advance_secs(duration_secs)?;
    write_session_outputs(&session, out_dir)?;
    eprintln!(
        "ran {duration_secs} s → energy {:.6} kWh, {} samples, status {}",
        report.energy_interval_kwh,
        report.samples_added,
        report.snapshot.grid_status
    );
    println!("{}", serde_json::to_string_pretty(&report.snapshot)?);
    Ok(())
}

fn session_resume(
    checkpoint: &Path,
    duration_secs: f64,
    out_dir: Option<&Path>,
    sample_period_secs: f64,
    commands_path: Option<&Path>,
) -> Result<()> {
    if duration_secs < 0.0 {
        bail!("duration-secs must be >= 0");
    }
    let mut session = Session::load_checkpoint(checkpoint)
        .with_context(|| format!("loading checkpoint {}", checkpoint.display()))?;
    session.set_sample_period_s(sample_period_secs);
    if session.phase() != energy_sim_runtime::SessionPhase::Running {
        session.start()?;
    }
    apply_commands_file(&mut session, commands_path)?;
    let report = session.advance_secs(duration_secs)?;
    let out = out_dir
        .map(PathBuf::from)
        .or_else(|| checkpoint.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    write_session_outputs(&session, &out)?;
    eprintln!(
        "resumed +{duration_secs} s → sim_time_s={}, energy_interval {:.6} kWh",
        report.snapshot.sim_time_s, report.energy_interval_kwh
    );
    println!("{}", serde_json::to_string_pretty(&report.snapshot)?);
    Ok(())
}

fn session_export(checkpoint: &Path, format: ExportFormat, out: &Path) -> Result<()> {
    let session = Session::load_checkpoint(checkpoint)
        .with_context(|| format!("loading checkpoint {}", checkpoint.display()))?;
    match format {
        ExportFormat::Csv => session.export_series_csv(out)?,
        ExportFormat::Jsonl => session.export_events_jsonl(out)?,
    }
    eprintln!("wrote {}", out.display());
    Ok(())
}

fn session_status(checkpoint: &Path) -> Result<()> {
    let session = Session::load_checkpoint(checkpoint)
        .with_context(|| format!("loading checkpoint {}", checkpoint.display()))?;
    let snap = session.snapshot();
    println!("{}", serde_json::to_string_pretty(&snap)?);
    Ok(())
}

fn apply_commands_file(session: &mut Session, path: Option<&Path>) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };
    let text =
        fs::read_to_string(path).with_context(|| format!("reading commands {}", path.display()))?;
    let commands: Vec<Command> = serde_json::from_str(&text)
        .with_context(|| format!("parsing commands JSON {}", path.display()))?;
    for cmd in commands {
        session.apply(cmd)?;
    }
    Ok(())
}

fn write_session_outputs(session: &Session, out_dir: &Path) -> Result<()> {
    let pkg = RunPackage::new(out_dir);
    pkg.write_all(session)
        .with_context(|| format!("writing run package {}", out_dir.display()))?;
    eprintln!(
        "wrote {}, {}, {}, {}",
        pkg.checkpoint_path().display(),
        pkg.events_path().display(),
        pkg.series_path().display(),
        pkg.manifest_path().display()
    );
    Ok(())
}
