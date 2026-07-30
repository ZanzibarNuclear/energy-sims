# Hydro Config Lab

Interactive **clean-slate** plant builder for [energy-sims](../..): place intake, lay penstock, position turbine, then run trials against the production engine.

Design: [`docs/hydro-config-lab.md`](../../docs/hydro-config-lab.md)  
Plan: [`docs/plans/hydro-config-lab.md`](../../docs/plans/hydro-config-lab.md)

## Status

**Lab-PR5 complete (PR1–PR5)** — Clean-slate build → compile → save/export → engine trials with ramp charts.  
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
  App.vue
  components/
    ServerStatus.vue
    ConfigToolbar.vue          # save / export / import
    SiteCanvas.vue             # construction tools + SVG plot
    SelectionPanel.vue
    PlantForm.vue
    SteadyPreview.vue
    TrialRunner.vue
    SeriesChart.vue
  lib/
    site.ts
    compileSite.ts             # geometry → plant (+ vitest)
    plantParams.ts
    labDocument.ts
    configStore.ts             # localStorage
    trialTypes.ts
    energySimClient.ts
```

### Suggested manual check (review)

1. Start server + `npm run dev`
2. Place intake high, turbine lower; optional bend
3. Confirm steady preview target kW
4. Save config; reload browser; open saved
5. Export plant → `cargo run -p energy-sim-cli -- hydro eval --config …`
6. Run spin-up trial; confirm power curve rises; Stop for spin-down
