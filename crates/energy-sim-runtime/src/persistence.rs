//! Durable checkpoints and run-directory packaging.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, RuntimeError};
use crate::export::{write_events_jsonl, write_series_csv};
use crate::session::Session;

/// Well-known filenames inside a run package directory.
pub const CHECKPOINT_FILE: &str = "checkpoint.json";
pub const EVENTS_FILE: &str = "events.jsonl";
pub const SERIES_FILE: &str = "series.csv";
pub const MANIFEST_FILE: &str = "manifest.json";

/// Metadata written beside checkpoint / history files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunManifest {
    pub schema_version: u32,
    pub kind: String,
    pub engine_version: String,
    pub sim_time_s: f64,
    pub phase: String,
    pub plant_id: String,
    pub sample_count: usize,
    pub event_count: usize,
    pub energy_generated_kwh: f64,
}

/// Paths for a run package directory.
#[derive(Debug, Clone)]
pub struct RunPackage {
    pub dir: PathBuf,
}

impl RunPackage {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn checkpoint_path(&self) -> PathBuf {
        self.dir.join(CHECKPOINT_FILE)
    }

    pub fn events_path(&self) -> PathBuf {
        self.dir.join(EVENTS_FILE)
    }

    pub fn series_path(&self) -> PathBuf {
        self.dir.join(SERIES_FILE)
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.dir.join(MANIFEST_FILE)
    }

    /// Ensure the package directory exists.
    pub fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.dir)?;
        Ok(())
    }

    /// Write checkpoint (atomic), events, series, and manifest from a session.
    pub fn write_all(&self, session: &Session) -> Result<()> {
        self.ensure_dir()?;
        atomic_write_json(&self.checkpoint_path(), &session_checkpoint_value(session)?)?;
        write_events_jsonl(self.events_path(), session.events())?;
        write_series_csv(self.series_path(), session.samples())?;
        let manifest = manifest_from_session(session);
        atomic_write_json(&self.manifest_path(), &serde_json::to_value(&manifest)?)?;
        Ok(())
    }

    /// Load a session from this package's checkpoint.
    pub fn load_session(&self) -> Result<Session> {
        let path = self.checkpoint_path();
        if !path.exists() {
            return Err(RuntimeError::Other(format!(
                "missing checkpoint at {}",
                path.display()
            )));
        }
        Session::load_checkpoint(path)
    }

    /// Basic package integrity checks (files present, checkpoint parseable).
    pub fn validate(&self) -> Result<RunManifest> {
        let ckpt = self.checkpoint_path();
        if !ckpt.exists() {
            return Err(RuntimeError::Other(format!(
                "package missing {CHECKPOINT_FILE}"
            )));
        }
        let session = Session::load_checkpoint(&ckpt)?;
        // Prefer on-disk manifest when present; otherwise derive.
        if self.manifest_path().exists() {
            let file = File::open(self.manifest_path())?;
            let manifest: RunManifest = serde_json::from_reader(file)?;
            if (manifest.sim_time_s - session.sim_time_s()).abs() > 1e-6 {
                return Err(RuntimeError::Other(
                    "manifest sim_time_s does not match checkpoint".into(),
                ));
            }
            return Ok(manifest);
        }
        Ok(manifest_from_session(&session))
    }
}

/// Write JSON to `path` via a temp file + rename for crash safety.
pub fn atomic_write_json(path: &Path, value: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = temp_sibling(path)?;
    {
        let mut file = File::create(&tmp)?;
        serde_json::to_writer_pretty(&mut file, value)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
    }
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        RuntimeError::Io(e)
    })?;
    Ok(())
}

fn temp_sibling(path: &Path) -> Result<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("data.json");
    let tmp = parent.join(format!(
        ".{name}.{}.tmp",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    Ok(tmp)
}

fn manifest_from_session(session: &Session) -> RunManifest {
    let snap = session.snapshot();
    RunManifest {
        schema_version: 1,
        kind: "energy-sim-run-manifest".into(),
        engine_version: crate::VERSION.to_string(),
        sim_time_s: snap.sim_time_s,
        phase: format!("{:?}", snap.phase).to_ascii_lowercase(),
        plant_id: snap.plant_id,
        sample_count: session.samples().len(),
        event_count: session.events().len(),
        energy_generated_kwh: snap.energy_generated_kwh,
    }
}

/// Build a serde value for checkpoint without going through filesystem.
fn session_checkpoint_value(session: &Session) -> Result<serde_json::Value> {
    // Reuse Session::save_checkpoint logic by writing to a temp buffer path is awkward;
    // call the same shape via a temp dir would work, but we expose save_checkpoint_to_value.
    session.checkpoint_value()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plant_json() -> String {
        std::fs::read_to_string("fixtures/plants/clearwater-diversion.json").unwrap_or_else(|_| {
            std::fs::read_to_string("../../fixtures/plants/clearwater-diversion.json").unwrap()
        })
    }

    #[test]
    fn package_write_load_mid_ramp() {
        let dir = std::env::temp_dir().join(format!("energy-sim-pkg-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let pkg = RunPackage::new(&dir);

        let mut session = Session::from_json(&plant_json()).unwrap();
        session.set_sample_period_s(1.0);
        session.start().unwrap();
        // Mid spin-up: not yet at target.
        session.advance_secs(10.0).unwrap();
        let power = session.snapshot().electrical_power_kw;
        let target = session.snapshot().target_electrical_power_kw;
        assert!(power < target * 0.95, "expected mid-ramp power={power} target={target}");
        let t = session.sim_time_s();
        let e = session.snapshot().energy_generated_kwh;

        pkg.write_all(&session).unwrap();
        assert!(pkg.checkpoint_path().exists());
        assert!(pkg.series_path().exists());
        assert!(pkg.events_path().exists());
        assert!(pkg.manifest_path().exists());

        let manifest = pkg.validate().unwrap();
        assert!((manifest.sim_time_s - t).abs() < 1e-9);

        let mut loaded = pkg.load_session().unwrap();
        assert!((loaded.sim_time_s() - t).abs() < 1e-9);
        assert!((loaded.snapshot().electrical_power_kw - power).abs() < 1e-12);
        assert!((loaded.snapshot().energy_generated_kwh - e).abs() < 1e-12);

        // Continue ramp from checkpoint.
        loaded.advance_secs(20.0).unwrap();
        assert!(loaded.snapshot().electrical_power_kw > power);
        assert!(loaded.sim_time_s() > t);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn atomic_write_creates_file() {
        let dir = std::env::temp_dir().join(format!("energy-sim-atomic-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("x.json");
        atomic_write_json(&path, &serde_json::json!({"ok": true})).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("ok"));
        // No leftover tmp files.
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("tmp"))
            .collect();
        assert!(leftovers.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
