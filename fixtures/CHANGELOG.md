# Fixture changelog (plant of record)

Record **why** Clearwater plant or station numbers changed when promoting a lab export.  
Process: [README.md](README.md) · design: [docs/design.md](../docs/design.md).

| Date | Documents | Change |
| --- | --- | --- |
| 2026-07-30 | `plants/clearwater-diversion.json` (+ session/grid) | Initial Clearwater Diversion plant + campus load registry (Stage 1). |
| 2026-08-05 | `stations/clearwater-diversion.json`, `grids/clearwater-diversion.json` | Drop separate “utility-station” naming; one plant of record is Clearwater Diversion. |

When updating:

1. Edit plant (and nested plant in the station document) together.
2. Run `./scripts/smoke-clearwater.sh`.
3. Add a row here with date, files, and a one-line reason (gameplay, teaching, bugfix).
