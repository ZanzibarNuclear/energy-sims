# Hydro Config Lab

Interactive **clean-slate** plant builder for [energy-sims](../..): place intake, lay penstock, position turbine, then run trials against the production engine.

Design: [`docs/hydro-config-lab.md`](../../docs/hydro-config-lab.md)  
Plan: [`docs/plans/hydro-config-lab.md`](../../docs/plans/hydro-config-lab.md)

## Status

**Lab-PR1** — Vue 3 + Vite scaffold, engine health banner, typed client. Construction tools, compile, save, and trials come next.

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
  App.vue                      # shell
  components/
    ServerStatus.vue           # health poll
    SiteCanvasPlaceholder.vue  # clean-slate canvas (tools → Lab-PR2)
  lib/
    energySimClient.ts         # REST + WS client (from clients/js)
```
