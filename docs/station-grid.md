# Station electrical grid

**Status:** Stub — filled in by PR4 (`feat(energy-sim): station bus, loads, surplus/shortage/brownout report`).

## Stage 1 model

- Sources (hydro first) attach generation to a **local bus**
- Loads draw from the bus
- Engine reports supply, demand, surplus/deficit, and **report-only** brownout/shortage
- No automatic load shedding in Stage 1 (host may react: dim lights, warn, etc.)

## Planned contents

- Bus state and balance semantics
- Load registry JSON shape
- Snapshot fields for control-room telemetry
- Multi-source attachment notes (solar, battery, fission, fusion later)

See [design.md](design.md) for architecture and key decisions.
