# API surface

**Status:** Stub — library/CLI documented as features land; remote service detailed in PR7.

## Layers (Stage 1 priority)

| Surface | Role | When |
| --- | --- | --- |
| **Library** (`energy-sim-core`, `energy-sim-runtime`) | In-process evaluation and sessions | PR2–PR4 |
| **CLI** (`energy-sim`) | Headless config → run → export | PR5 |
| **HTTP + WebSocket** (`energy-sim-server`) | Remote control + live telemetry | PR7 |
| **WASM** (`energy-sim-wasm`) | Optional embed path | PR9 (optional) |

## Preferred host path

Game and multi-player hosts should prefer the **remote service** (REST for control, WebSocket for live snapshots). Library + CLI always remain available for tests and tooling.

## Planned contents

- Library module map and key types
- CLI commands and file formats (`checkpoint.json`, `events.jsonl`, `series.csv`)
- REST endpoints and WebSocket message shapes
- Error and versioning notes

See [design.md](design.md) for sketches and decisions.
