# API surface

**Status:** Library + CLI + REST/WebSocket server + WASM (Stage 1; long-lived WASM session next).  
**Host strategy:** same session contract for **WASM (game alpha on device)** and **HTTP/WS (lab today; hosted game later)**. See [design.md](design.md).

## Layers

| Surface | Role |
| --- | --- |
| **Library** (`energy-sim-core`, `energy-sim-runtime`) | In-process evaluation and sessions |
| **CLI** (`energy-sim`) | Headless config → run → export |
| **HTTP + WebSocket** (`energy-sim-server`) | Remote control + live telemetry |
| **WASM** (`energy-sim-wasm`) | In-browser / embed path for game alpha and offline hosts |

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

## Remote service (`energy-sim-server`)

```bash
cargo run -p energy-sim-server -- --listen 127.0.0.1:8787
```

### REST

| Method | Path | Body | Response |
| --- | --- | --- | --- |
| GET | `/health` | — | `{ ok, engine }` |
| POST | `/v1/sessions` | plant or session JSON | `{ sessionId, snapshot }` |
| GET | `/v1/sessions/{id}` | — | `Snapshot` |
| POST | `/v1/sessions/{id}/start` | — | `Snapshot` |
| POST | `/v1/sessions/{id}/stop` | — | `Snapshot` |
| POST | `/v1/sessions/{id}/advance` | `{ durationSecs, commands? }` | `AdvanceReport` |
| POST | `/v1/sessions/{id}/tick` | `{ dtSecs }` | `Snapshot` |
| POST | `/v1/sessions/{id}/commands` | `{ commands: Command[] }` | `Snapshot` |
| GET | `/v1/sessions/{id}/history?fromSecs=&toSecs=` | — | `{ events, samples }` |
| POST | `/v1/sessions/{id}/checkpoint` | — | full checkpoint JSON |

### WebSocket live channel

`WS /v1/sessions/{id}/live`

**Server → client**

```json
{ "type": "snapshot", "snapshot": { "...": "..." } }
{ "type": "error", "message": "..." }
{ "type": "pong" }
```

**Client → server**

```json
{ "type": "tick", "dtSecs": 1.0 }
{ "type": "advance", "durationSecs": 60 }
{ "type": "command", "command": { "type": "set_load", "id": "lighting.main", "drawing": true } }
{ "type": "ping" }
```

In-memory sessions only (Stage 1). Auth and multi-tenant storage are later.

## WASM (`energy-sim-wasm`)

**Primary short-term path for Atomic Adventures:** run the engine on the player device so static deploys do not need a sim server. The **remote service** remains the lab path today and the long-term hosted path.

```sh
# requires wasm-pack and rustup target wasm32-unknown-unknown
wasm-pack build crates/energy-sim-wasm --target web
```

### Current exports (Stage 1 minimal)

| JS name | Role |
| --- | --- |
| `version()` | Banner string |
| `evaluateHydro(plantJson, operatorJson?)` | Steady-state hydro eval |
| `runSession(configJson, durationSecs, commandsJson?)` | One-shot start + advance; returns `AdvanceReport` |
| `sessionSnapshot(configJson, start)` | Snapshot without long advance |

### Planned exports (game-ready — see [plans/next.md](plans/next.md))

Long-lived session handle aligned with the HTTP API:

| Operation | Role |
| --- | --- |
| create / free session | Hold runtime state in WASM memory |
| start / stop | Phase |
| advance / tick | Sim time |
| commands | Hydro operator + loads |
| snapshot / history | Console + charts |
| checkpoint (optional) | Serialize / restore |

Hosts should use a thin **adapter** so switching WASM ↔ HTTP does not rewrite control-room or holo modules.
