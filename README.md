# energy-sims

Energy production simulators for **Zanzibar's World of Energy** / Atomic Adventures.

Rust, headless-first: hydropower and a local station grid in Stage 1, with room for solar, battery, fission, fusion, and real-world component catalogs later.

**Repository:** [github.com/ZanzibarNuclear/energy-sims](https://github.com/ZanzibarNuclear/energy-sims)

## Status

**Stage 1 engine implemented** (hydro physics, ramps, station grid, CLI, file packages, REST + WebSocket server, JS client, WASM). **Hydro config lab MVP shipped** (`apps/hydro-config-lab/`). Architecture and game integration design: [docs/design.md](docs/design.md). Remaining work: [docs/plans/next.md](docs/plans/next.md) (WASM long-lived session, grid presentation, then Atomic Adventures wiring).

## Quick start

```sh
cargo test
cargo run -p energy-sim-cli -- version

# Steady-state hydro eval
cargo run -p energy-sim-cli -- hydro eval --config fixtures/plants/clearwater-diversion.json

# Session: 120 s with loads → checkpoint.json, series.csv, events.jsonl
cargo run -p energy-sim-cli -- session run \
  --config fixtures/stations/clearwater-diversion.json \
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

Clearwater plant-of-record smoke (after fixture changes):

```sh
./scripts/smoke-clearwater.sh
```

## Workspace layout

```text
energy-sims/
  Cargo.toml                 # workspace
  crates/
    energy-sim-core/         # physics and types
    energy-sim-runtime/      # sessions, ramps, grid, history
    energy-sim-cli/          # headless operator (bin: energy-sim)
    energy-sim-server/       # REST + WebSocket
    energy-sim-wasm/         # optional embed path
  clients/js/                # thin browser client
  apps/hydro-config-lab/     # Vue config lab (prototyping UI)
  fixtures/                  # example plant + load configs (JSON)
  examples/                  # sample runs / export samples
  docs/
    design.md                # architecture (Stage 1 complete)
    hydro-physics.md         # equations and parameters
    station-grid.md          # bus / loads
    api.md                   # library, CLI, HTTP / WS
    hydro-config-lab.md      # config lab (MVP shipped)
    plans/next.md            # game-readiness plan
```

Optional WASM: `wasm-pack build crates/energy-sim-wasm --target web`.

### Hydro config lab (Vue)

```sh
# terminal 1 — engine
cargo run -p energy-sim-server -- --listen 127.0.0.1:8787

# terminal 2 — UI
cd apps/hydro-config-lab && npm install && npm run dev
```

See [apps/hydro-config-lab/README.md](apps/hydro-config-lab/README.md).

## Stage 1 (shipped)

- Hydro power from \(P = \eta \rho g Q H_\mathrm{net}\) with configurable head, flow, penstock, losses, efficiency
- **Ramp-up / ramp-down** (turbine spin-up/spin-down) so graphs show transitions
- Session lifecycle: start / stop / interval / tick (sim time in **seconds**)
- Station grid: sources feed a bus, loads draw, **report-only** brownout/shortage
- Operational data: JSON checkpoint, **CSV** series, JSONL events
- Headless **CLI** and **REST + WebSocket** service for remote hosts

## Docs

| Doc | Description |
| --- | --- |
| [docs/design.md](docs/design.md) | Architecture, integration, Clearwater workflow |
| [docs/hydro-physics.md](docs/hydro-physics.md) | Hydro equations and parameters |
| [docs/station-grid.md](docs/station-grid.md) | Station bus and balance semantics |
| [docs/api.md](docs/api.md) | Library, CLI, HTTP/WS, WASM |
| [docs/hydro-config-lab.md](docs/hydro-config-lab.md) | Config lab (MVP shipped) |
| [docs/plans/next.md](docs/plans/next.md) | Remaining game-readiness work |

## License

[GNU GPL v3 or later](LICENSE).
