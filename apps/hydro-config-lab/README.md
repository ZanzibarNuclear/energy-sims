# Hydro Config Lab

Interactive **clean-slate** plant builder for [energy-sims](../..): place intake, lay penstock, position turbine, then run trials against the production engine.

Design: [`docs/hydro-config-lab.md`](../../docs/hydro-config-lab.md)  
Plan: [`docs/plans/hydro-config-lab.md`](../../docs/plans/hydro-config-lab.md)

## Status

**Workflow UI** — three tabs: **Layout** → **Equipment** → **Run**. File actions in the ☰ menu.  
Paused before station loads (Lab-PR6).

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
    SiteCanvas.vue             # Layout tab (default intake+turbine)
    PlantForm.vue              # Equipment tab
    SteadyPreview.vue
    ServerStatus.vue           # Run tab
    TrialRunner.vue
    SeriesChart.vue
  lib/ …
```

### Workflow

1. **Layout** — place intake / penstock / turbine on the grid  
2. **Equipment** — stream flow, diameter, friction, η, operator; steady preview  
3. **Run** — engine status, ▶ Play / ⏹ Stop, power & speed charts  

File menu (☰): New, Save, Export plant/lab JSON, Import.
