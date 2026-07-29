# energy-sims

Energy production simulators for **Zanzibar's World of Energy** / Atomic Adventures.

Rust, headless-first: hydropower and a local station grid in Stage 1, with room for solar, battery, fission, fusion, and real-world component catalogs later.

**Repository:** [github.com/ZanzibarNuclear/energy-sims](https://github.com/ZanzibarNuclear/energy-sims)

## Status

Design complete — ready to implement. See [docs/design.md](docs/design.md).

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

## Intended layout

```text
energy-sims/
  Cargo.toml                 # workspace
  crates/
    energy-sim-core/         # physics and types
    energy-sim-runtime/      # sessions, ramps, grid, history
    energy-sim-cli/          # headless operator
    energy-sim-server/       # REST + WebSocket (when ready)
    energy-sim-wasm/         # optional embed path
  fixtures/
  docs/
    design.md
```

## License

See [LICENSE](LICENSE).
