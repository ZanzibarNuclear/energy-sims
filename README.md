# energy-sims

Energy production simulators for **Zanzibar's World of Energy** / Atomic Adventures.

Rust, headless-first: hydropower and a local station grid in Stage 1, with room for solar, battery, fission, fusion, and real-world component catalogs later.

**Repository:** [github.com/ZanzibarNuclear/energy-sims](https://github.com/ZanzibarNuclear/energy-sims)

## Status

Workspace scaffolded. Physics and runtime land in subsequent PRs. See [docs/design.md](docs/design.md).

## Quick start

```sh
cargo test
cargo run -p energy-sim-cli -- version

# Steady-state hydro eval
cargo run -p energy-sim-cli -- hydro eval --config fixtures/plants/upper-penstock.json

# Session: 120 s with loads → checkpoint.json, series.csv, events.jsonl
cargo run -p energy-sim-cli -- session run \
  --config fixtures/stations/utility-station.json \
  --duration-secs 120 \
  --out-dir ./run-demo/ \
  --commands examples/commands-lights-ev.json
```

```sh
# Remote API (REST + WebSocket)
cargo run -p energy-sim-server -- --listen 127.0.0.1:8787
# POST /v1/sessions  GET /v1/sessions/{id}  WS /v1/sessions/{id}/live
```

Requires a recent stable Rust toolchain (MSRV **1.76**). More workflows in [examples/README.md](examples/README.md).

## Workspace layout

```text
energy-sims/
  Cargo.toml                 # workspace
  crates/
    energy-sim-core/         # physics and types
    energy-sim-runtime/      # sessions, ramps, grid, history
    energy-sim-cli/          # headless operator (bin: energy-sim)
  fixtures/                  # example plant + load configs (JSON)
  examples/                  # sample runs / export samples
  docs/
    design.md                # architecture and PR plan
    hydro-physics.md         # equations (stub → PR2)
    station-grid.md          # bus / loads (stub → PR4)
    api.md                   # library, CLI, HTTP (stub → PR5/PR7)
```

Also in the workspace: `energy-sim-server` (REST + WebSocket) and optional `energy-sim-wasm` (`wasm-pack build crates/energy-sim-wasm --target web`).

## Stage 1 (planned)

- Hydro power from \(P = \eta \rho g Q H_\mathrm{net}\) with configurable head, flow, penstock, losses, efficiency
- **Ramp-up / ramp-down** (e.g. turbine spin-up) so graphs show transitions
- Session lifecycle: start / stop / interval / tick (sim time in **seconds**)
- Station grid: sources feed a bus, loads draw, **report-only** brownout/shortage
- Operational data: JSON checkpoint, **CSV** series, JSONL events
- Headless **CLI**; later **REST + WebSocket** service for game control room

## Docs

| Doc | Description |
| --- | --- |
| [docs/design.md](docs/design.md) | Architecture, goals, API sketches, PR plan |
| [docs/hydro-physics.md](docs/hydro-physics.md) | Hydro equations and parameters |
| [docs/station-grid.md](docs/station-grid.md) | Station bus and balance semantics |
| [docs/api.md](docs/api.md) | Library, CLI, and remote API surface |

## License

[GNU GPL v3 or later](LICENSE).
