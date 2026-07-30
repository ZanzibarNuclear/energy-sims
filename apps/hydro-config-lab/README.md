# Hydro Config Lab

Interactive **clean-slate** plant builder for [energy-sims](../..): place intake, lay penstock, position turbine, then run trials against the production engine.

Design: [`docs/hydro-config-lab.md`](../../docs/hydro-config-lab.md)  
Plan: [`docs/plans/hydro-config-lab.md`](../../docs/plans/hydro-config-lab.md)

## Status

**Lab-PR2** — Clean-slate site canvas: place intake, penstock bends, turbine; select/drag/delete.  
Next: Lab-PR3 compile → plant + engine preview.

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

## Layout

```text
src/
  App.vue
  components/
    ServerStatus.vue
    SiteCanvas.vue             # construction tools + SVG plot
    SelectionPanel.vue         # edit selected point
  lib/
    site.ts                    # site model
    energySimClient.ts         # REST + WS client
```
