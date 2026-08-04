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

See `fixtures/grids/utility-station.json`. Fields:

| Field | Role |
| --- | --- |
| `id` | Stable load id (e.g. `ev-charge.port-1`) |
| `label` | Human-readable name |
| `ratingW` | Demand when drawing (watts) |
| `priority` | `critical` / `normal` / `deferrable` (hints for later shed policies) |
| `initiallyDrawing` | Starting state |
| `packageId` | Optional catalog hook (ignored for now) |

## Session composition

Full station document (`fixtures/stations/utility-station.json`):

```json
{
  "schemaVersion": 1,
  "kind": "energy-session",
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

| Aggregate (on `Snapshot` today) | Per-load (game-ready expansion) |
| --- | --- |
| `availableGenerationKw`, `totalLoadKw`, `marginKw` | Load id, label, rating, priority, drawing |
| `busEnergized`, `gridStatus` | Same ids as game circuit bindings |
| Events for brownout enter/clear | History for alarms / charts |

Per-load rows on the host-facing snapshot (or companion DTO) are tracked in [plans/next.md](plans/next.md). Until then, hosts that only need totals can use aggregate fields.

## Multi-source later

`StationGrid::balance(available_generation_kw)` takes a single generation number today. Later sessions can sum N sources before calling balance—no API redesign required for the bus.

See [design.md](design.md) for architecture and key decisions.
