# Fixtures

Example plant and load configuration JSON for headless runs, tests, and the **game plant of record**.

**Naming**

| Name | Role |
| --- | --- |
| **Clearwater Run** | Stream (story / geography) |
| **Clearwater Diversion** | Hydro plant (`hydro-plant`) |
| **Clearwater Station** | Campus facility session: plant + bus (`energy-session`) |

Architecture / promote workflow: [docs/design.md](../docs/design.md).  
Changelog of fixture promotions: [CHANGELOG.md](CHANGELOG.md).

## Clearwater (plant of record)

| File | Kind | Use |
| --- | --- | --- |
| `plants/clearwater-diversion.json` | `hydro-plant` | Plant-only (lab export target, `hydro eval`, teaching) |
| `stations/clearwater-station.json` | `energy-session` | Plant **+** bus loads for control-room / station sessions |
| `grids/clearwater-station.json` | `station-grid` | Load registry alone (authoring/tests; also nested in the session file) |

Ids/labels: plant `clearwater-diversion` / “Clearwater Diversion”; station and grid `clearwater-station` / “Clearwater Station”. Keep the nested `plant` in the session file in sync with `plants/clearwater-diversion.json` when promoting lab exports.

## Other plants

| File | Notes |
| --- | --- |
| `plants/ideal-teaching.json` | Zero-loss elevation-only teaching case |

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
    + nested plant in stations/clearwater-station.json
    + fixtures/CHANGELOG.md row
       │
       ▼
  Game loads station document through EnergySim adapter (WASM or HTTP)
```

### Steps

1. **Tune** in `apps/hydro-config-lab` → **Export** plant JSON (engine schema).
2. **Update fixtures** — replace `plants/clearwater-diversion.json` and the `plant` object inside `stations/clearwater-station.json` so they stay in sync. Keep grid load ids unchanged unless the game migrates with you.
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
  --config fixtures/stations/clearwater-station.json \
  --duration-secs 30 \
  --out-dir ./target/clearwater-smoke/ \
  --commands examples/commands-lights-ev.json
```

Keep committed fixtures small and teaching-oriented; ignore ad-hoc run outputs via the root `.gitignore` (`target/` is already ignored).
