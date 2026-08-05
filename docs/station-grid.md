# Station electrical grid

**Status:** Implemented in `energy-sim-runtime` (PR4).  
**Game use:** control-room **grid terminal** + facility loads; see [design.md](design.md) host integration.

## Stage 1 model

```text
  Hydro plant ──► generation (kW, ramped actual)
                      │
                      ▼
              ┌───────────────┐
   loads ───► │  station bus  │ ──► balance, surplus/deficit, brownout
              └───────────────┘
```

- Sources attach **available generation** (hydro actual electrical power today).
- Loads draw from the bus when `drawing = true`.
- Engine reports supply, demand, margin, and status.
- **Brownout policy = report-only:** no automatic load shedding.
- The grid is **more than brownout**: it is the station’s named loads, ratings, drawing state, and bus margin—the control console’s second terminal (alongside hydro sensors).

## Status values

| Status | Meaning |
| --- | --- |
| `ok` | Balanced or idle (no meaningful surplus/deficit) |
| `surplus` | Generation > drawing load |
| `shortage` | Load exceeds generation by less than the brownout threshold |
| `brownout` | Deficit ≥ `brownout.deficitThresholdW` (default 1 W) |

`busEnergized` is true when available generation > 0. Under brownout the bus may still be “live” so the host can dim lights / warn—Stage 1 does not kill circuits for you.

## Load registry JSON

See `fixtures/grids/clearwater-diversion.json`. Fields:

| Field | Role |
| --- | --- |
| `id` | Stable load id (e.g. `ev-charge.port-1`) |
| `label` | Human-readable name |
| `ratingW` | Demand when drawing (watts) |
| `priority` | `critical` / `normal` / `deferrable` (hints for later shed policies) |
| `initiallyDrawing` | Starting state |
| `packageId` | Optional catalog hook (ignored for now) |

## Session composition

Full session document for Clearwater Diversion (`fixtures/stations/clearwater-diversion.json`): plant + its bus. This is the same plant of record as `plants/clearwater-diversion.json`, not a second facility.

```json
{
  "schemaVersion": 1,
  "kind": "energy-session",
  "id": "clearwater-diversion",
  "plant": { "...": "hydro-plant" },
  "grid": { "...": "station-grid" }
}
```

Bare hydro plant JSON still works (no loads → margin = full generation).

## Commands

```rust
session.apply(Command::SetLoad { id: "lighting.main".into(), drawing: true })?;
```

Unknown load ids error. Load changes emit events; transitions into/out of brownout emit `brownout_entered` / `brownout_cleared`.

## Control-console presentation

Every `Snapshot` includes both aggregates and a **load table** for the grid terminal:

| Field | Role |
| --- | --- |
| `availableGenerationKw`, `totalLoadKw`, `marginKw` | Bus balance |
| `busEnergized`, `gridStatus` | Status strip / brownout presentation |
| `loads[]` | Per-load rows: `id`, `label`, `ratingW`, `priority`, `drawing` |

Load `id` values match station fixture circuit ids (`lighting.main`, `ev-charge.port-1`, …). Toggling drawing is still via `Command::SetLoad`; the table is presentation only (no auto-shed).

## Multi-source later

`StationGrid::balance(available_generation_kw)` takes a single generation number today. Later sessions can sum N sources before calling balance—no API redesign required for the bus.

See [design.md](design.md) for architecture and key decisions.
