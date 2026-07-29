//! File export: CSV series and JSONL events.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::error::Result;
use crate::history::{Event, Sample};

/// Write operational series as CSV (header + rows).
pub fn write_series_csv(path: impl AsRef<Path>, samples: &[Sample]) -> Result<()> {
    let file = File::create(path)?;
    let mut w = BufWriter::new(file);
    writeln!(
        w,
        "sim_time_s,electrical_power_kw,target_electrical_power_kw,turbine_speed_rpm,target_turbine_speed_rpm,flow_m3s,net_head_m,available_generation_kw,total_load_kw,margin_kw,grid_status,energy_generated_kwh"
    )?;
    for s in samples {
        writeln!(
            w,
            "{},{},{},{},{},{},{},{},{},{},{},{}",
            s.sim_time_s,
            s.electrical_power_kw,
            s.target_electrical_power_kw,
            s.turbine_speed_rpm,
            s.target_turbine_speed_rpm,
            s.flow_m3s,
            s.net_head_m,
            s.available_generation_kw,
            s.total_load_kw,
            s.margin_kw,
            s.grid_status,
            s.energy_generated_kwh
        )?;
    }
    w.flush()?;
    Ok(())
}

/// Append or write events as JSONL (one JSON object per line).
pub fn write_events_jsonl(path: impl AsRef<Path>, events: &[Event]) -> Result<()> {
    let file = File::create(path)?;
    let mut w = BufWriter::new(file);
    for e in events {
        serde_json::to_writer(&mut w, e)?;
        writeln!(w)?;
    }
    w.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{EventKind, Sample};

    #[test]
    fn csv_round_trip_header() {
        let dir = std::env::temp_dir().join(format!("energy-sim-csv-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("series.csv");
        let samples = vec![Sample {
            sim_time_s: 1.0,
            electrical_power_kw: 2.5,
            target_electrical_power_kw: 3.0,
            turbine_speed_rpm: 500.0,
            target_turbine_speed_rpm: 1000.0,
            flow_m3s: 0.04,
            net_head_m: 24.0,
            available_generation_kw: 2.5,
            total_load_kw: 1.0,
            margin_kw: 1.5,
            grid_status: "surplus".into(),
            energy_generated_kwh: 0.001,
        }];
        write_series_csv(&path, &samples).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("sim_time_s,"));
        assert!(text.contains("2.5"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn jsonl_writes_lines() {
        let dir = std::env::temp_dir().join(format!("energy-sim-jsonl-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("events.jsonl");
        let events = vec![Event {
            sim_time_s: 0.0,
            kind: EventKind::Started,
            message: Some("go".into()),
            detail: None,
        }];
        write_events_jsonl(&path, &events).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("started") || text.contains("Started"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
