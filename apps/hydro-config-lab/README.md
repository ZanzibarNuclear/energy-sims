# Hydro Config Lab

Interactive **clean-slate** plant builder for [energy-sims](../..): place intake, lay penstock, position turbine, then run trials against the production engine.

## Status

**MVP shipped** — four tabs: **Layout** → **Equipment** → **Calculations** → **Run**.  
**Save** keeps named configs in this browser; **Export / Import** for JSON files.  
Design: [`docs/hydro-config-lab.md`](../../docs/hydro-config-lab.md). Remaining work: [`docs/plans/next.md`](../../docs/plans/next.md).

## Prerequisites

- Node 20+ (22 OK)
- Rust toolchain for the engine server

## Run

Terminal 1 — engine:

```sh
# from repo root
cargo run -p energy-sim-server -- --listen 127.0.0.1:8787
```

Terminal 2 — lab UI:

```sh
cd apps/hydro-config-lab
npm install   # first time
npm run dev
```

Open the URL Vite prints (default `http://localhost:5173`). The header banner shows whether the engine is reachable.

### Engine URL

Default: `http://127.0.0.1:8787`

Override:

```sh
VITE_ENERGY_SIM_URL=http://127.0.0.1:8787 npm run dev
```

## Scripts

| Script | Purpose |
| --- | --- |
| `npm run dev` | Vite dev server |
| `npm run build` | Typecheck + production build |
| `npm run preview` | Preview production build |
| `npm test` | Compile math unit tests (vitest) |

## Layout

```text
src/
  App.vue                      # tab workflow shell
  components/
    AppMenu.vue                # ☰ save / export / import
    SiteCanvas.vue             # Layout tab
    PlantForm.vue              # Equipment tab
    SteadyPreview.vue          # Calculations tab
    PowerEquation.vue
    ServerStatus.vue           # Run tab
    TrialRunner.vue
    SeriesChart.vue
  lib/ …
```

### Workflow

1. **Layout** — place intake / penstock / turbine on the grid  
2. **Equipment** — flow, diameter, overall η (advanced overrides available)  
3. **Calculations** — steady preview, head-loss breakdown, power equation  
4. **Run** — engine status, timed trial, power & speed charts  

File menu (☰): New, Save / Save as (browser), Open saved list, Export JSON, Import JSON.

### Promote a plant to fixtures (Clearwater workflow)

1. Export plant JSON from the lab.  
2. Smoke: `cargo run -p energy-sim-cli -- hydro eval --config <export.json>`.  
3. PR into `fixtures/plants/` (and nested plant in `fixtures/stations/` if the station document should match).  
4. Game loads the fixture through the EnergySim adapter — see [docs/design.md](../../docs/design.md).
