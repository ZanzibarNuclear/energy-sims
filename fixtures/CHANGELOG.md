# Fixture changelog (plant of record)

Record **why** Clearwater plant or station numbers changed when promoting a lab export.  
Process: [README.md](README.md) · design: [docs/design.md](../docs/design.md).

| Date | Documents | Change |
| --- | --- | --- |
| 2026-07-30 | `plants/clearwater-diversion.json`, `stations/utility-station.json` | Initial campus diversion plant + utility station load registry (Stage 1). |

When updating:

1. Edit plant (and nested plant in the station document) together.
2. Run `./scripts/smoke-clearwater.sh`.
3. Add a row here with date, files, and a one-line reason (gameplay, teaching, bugfix).
