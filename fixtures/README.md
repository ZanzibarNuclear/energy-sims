# Fixtures

Example plant and load configuration JSON for headless runs, tests, and the **game plant of record**.

Architecture / promote workflow: [docs/design.md](../docs/design.md) (Clearwater plant-of-record section).  
Changelog of fixture promotions: [CHANGELOG.md](CHANGELOG.md).

## Plants

| File | Notes |
| --- | --- |
| `plants/clearwater-diversion.json` | Campus diversion on **Clearwater Run** — **primary plant** for Atomic Adventures Part I |
| `plants/ideal-teaching.json` | Zero-loss elevation-only teaching case |

## Grids & stations

| File | Notes |
| --- | --- |
| `grids/utility-station.json` | Standalone load registry |
| `stations/utility-station.json` | Composite session (Clearwater plant + grid) — **primary station document** for control-room sessions |

### Stable load ids (do not rename without a game migration)

| Id | Role |
| --- | --- |
| `lighting.main` | Main building lights |
| `holo-reader.library` | Library holo-reader |
| `ev-charge.port-1` | EV charge port |
| `kitchen.appliance` | Kitchen outlet circuit |

The game must bind circuits to these ids via the EnergySim adapter. Do not hard-code physics constants in the game repo.

## Promoting a lab export (Clearwater workflow)

```text
  Lab (tune site + equipment)
       │ Export plant JSON (engine schema)
       ▼
  ./scripts/smoke-clearwater.sh   # after updating fixtures, or smoke old plant first
       │
       ▼
  PR: plants/clearwater-diversion.json
    + nested plant in stations/utility-station.json
    + fixtures/CHANGELOG.md row
       │
       ▼
  Game loads station document through EnergySim adapter (WASM or HTTP)
```

### Steps

1. **Tune** in `apps/hydro-config-lab` → **Export** plant JSON (engine schema).
2. **Update fixtures** — replace `plants/clearwater-diversion.json` and the `plant` object inside `stations/utility-station.json` so they stay in sync. Keep grid load ids unchanged unless the game migrates with you.
3. **Smoke** from repo root:
   ```sh
   ./scripts/smoke-clearwater.sh
   # optional: ./scripts/smoke-clearwater.sh --out-dir /tmp/cw-smoke
   ```
   This runs `hydro eval` on the plant and a short `session run` on the station (lights + holo + EV commands).
4. **Changelog** — add a row to [CHANGELOG.md](CHANGELOG.md) describing what changed and why.
5. **PR** — review numbers (head, flow, η, rated kW) against the story / holo lesson.
6. **Game** — consume the station JSON through `createWasmBackend` / `createHttpBackend`; do not fork constants into Vue.

### Manual CLI (same checks as the script)

```sh
cargo run -p energy-sim-cli -- hydro eval --config fixtures/plants/clearwater-diversion.json

cargo run -p energy-sim-cli -- session run \
  --config fixtures/stations/utility-station.json \
  --duration-secs 30 \
  --out-dir ./target/clearwater-smoke/ \
  --commands examples/commands-lights-ev.json
```

Keep committed fixtures small and teaching-oriented; ignore ad-hoc run outputs via the root `.gitignore` (`target/` is already ignored).
