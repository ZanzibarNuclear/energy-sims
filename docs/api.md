# API surface

**Status:** Library + CLI live (PR2–PR5). Remote service in PR7.

## Layers

| Surface | Role |
| --- | --- |
| **Library** (`energy-sim-core`, `energy-sim-runtime`) | In-process evaluation and sessions |
| **CLI** (`energy-sim`) | Headless config → run → export |
| **HTTP + WebSocket** (`energy-sim-server`) | Remote control + live telemetry (PR7) |
| **WASM** (`energy-sim-wasm`) | Optional embed path (PR9) |

## Library (Rust)

```rust
use energy_sim_core::{evaluate_plant, HydroPlantConfig};
use energy_sim_runtime::{Command, Session};

let plant = HydroPlantConfig::from_json(&json)?;
let eval = evaluate_plant(&plant)?;

let mut session = Session::from_json(&station_json)?;
session.start()?;
session.apply(Command::SetLoad { id: "lighting.main".into(), drawing: true })?;
let report = session.advance_secs(3600.0)?;
session.save_checkpoint("checkpoint.json")?;
session.export_series_csv("series.csv")?;
```

Key types: `HydroPlantConfig`, `OperatorInputs`, `HydroEvaluation`, `Session`, `Snapshot`, `StationGrid`, `Command`.

## CLI

```bash
energy-sim version
energy-sim hydro eval --config plant.json [--gate 1] [--debris 0] [--leakage 0] [--offline]
energy-sim session run --config station.json --duration-secs 3600 --out-dir ./run/ [--commands cmds.json]
energy-sim session resume --checkpoint ./run/checkpoint.json --duration-secs 1800
energy-sim session export --checkpoint ./run/checkpoint.json --format csv|jsonl --out path
energy-sim session status --checkpoint ./run/checkpoint.json
```

### Output files (`session run` / `resume`)

| File | Format | Contents |
| --- | --- | --- |
| `checkpoint.json` | JSON | Full session state for resume |
| `events.jsonl` | JSONL | Append-friendly event log |
| `series.csv` | CSV | Time series including ramps |

`--commands` is a JSON array of `Command` objects, e.g. `examples/commands-lights-ev.json`.

## Remote service

REST + WebSocket shapes land with PR7. Sketch in [design.md](design.md).
