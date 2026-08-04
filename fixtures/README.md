# Fixtures

Example plant and load configuration JSON for headless runs, tests, and the **game plant of record**.

Architecture / promote workflow: [docs/design.md](../docs/design.md) (Clearwater plant-of-record section).

## Plants

| File | Notes |
| --- | --- |
| `plants/clearwater-diversion.json` | Campus diversion on **Clearwater Run** (with penstock losses) — **primary plant** for Atomic Adventures Part I |
| `plants/ideal-teaching.json` | Zero-loss elevation-only teaching case |

## Grids & stations

| File | Notes |
| --- | --- |
| `grids/utility-station.json` | Standalone load registry |
| `stations/utility-station.json` | Composite session (Clearwater plant + grid) — **primary station document** for control-room sessions |

Load ids in the station grid (`lighting.main`, `holo-reader.library`, `ev-charge.port-1`, …) should stay stable once the game binds circuits to them.

## Promoting a lab export (Clearwater workflow)

1. Tune in `apps/hydro-config-lab` and **Export** plant JSON (engine schema).
2. Smoke locally:
   ```sh
   cargo run -p energy-sim-cli -- hydro eval --config path/to/export.json
   cargo run -p energy-sim-cli -- session run \
     --config fixtures/stations/utility-station.json \
     --duration-secs 60 \
     --out-dir ./run-smoke/
   ```
   (After updating the plant, nest the new plant object into `stations/utility-station.json` or re-export a full station when the lab supports it.)
3. Open a PR that updates `plants/clearwater-diversion.json` and the nested plant in `stations/utility-station.json` together.
4. Note the change briefly in the PR description (what head/flow/η changed and why). Optional later: a `contentVersion` field on the document.
5. Game consumes the fixture through the EnergySim adapter — do not fork physics constants into the game repo.

Keep committed fixtures small and teaching-oriented; ignore ad-hoc run outputs via the root `.gitignore`.
