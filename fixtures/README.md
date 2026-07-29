# Fixtures

Example plant and load configuration JSON for headless runs and tests.

## Plants

| File | Notes |
| --- | --- |
| `plants/clearwater-diversion.json` | Campus diversion on **Clearwater Run** (with penstock losses) |
| `plants/ideal-teaching.json` | Zero-loss elevation-only teaching case |

## Grids & stations

| File | Notes |
| --- | --- |
| `grids/utility-station.json` | Standalone load registry |
| `stations/utility-station.json` | Composite session (plant + grid) |

Keep committed fixtures small and teaching-oriented; ignore ad-hoc run outputs via the root `.gitignore`.
