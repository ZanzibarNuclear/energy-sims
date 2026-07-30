# Plan: Hydro Config Lab

**Status:** Lab-PR4 done; next Lab-PR5 (trials)  
**Date:** 2026-07-30  
**Design:** [../hydro-config-lab.md](../hydro-config-lab.md)  
**Depends on:** Stage 1 engine (shipped) — server + plant/session JSON  
**Priority:** **Next product build** (ahead of database, catalog, and game wiring)

---

## Intent

Ship a Vue-based **config + trial** tool that authors plant geometry and parameters, runs them through `energy-sim-server`, and exports fixtures for the game and lesson experiments.

**Default UX:** clean-slate **site construction** — the user places an intake, lays the penstock, and positions the turbine. Not a preloaded plant form with sliders for everything.

Stack for MVP:

- **Vue 3 + Vite + TypeScript**
- **energy-sim-server** on localhost (REST + WebSocket)
- Local named configs + JSON import/export
- **No Tauri / Electron required** for first usable version (Tauri shell is a later optional PR)

**Sequencing notes**

- **Game control-room wiring waits** until config prototyping has settled the engine.
- **Tests:** only where they prevent real regressions (e.g. site→plant compile). No tests for appearance.
- **Station grid / loads:** not in the clean-slate MVP; add only if plant trials need brownout storytelling later.

---

## PR breakdown

### Lab-PR1 — Scaffold lab app ✅ done

- **Title:** `feat(lab): scaffold Vue Vite hydro-config-lab`
- **Affects:** `apps/hydro-config-lab/`, root README pointer, lab README, gitignore
- **Depends on:** none (engine already exists)
- **Description:**
  - Vite + Vue 3 + TS app under `apps/hydro-config-lab/`
  - Env `VITE_ENERGY_SIM_URL` default `http://127.0.0.1:8787`
  - Health check banner (server reachable / not) with retry
  - Typed port of `clients/js` → `src/lib/energySimClient.ts`
  - Shell UI: clean-slate placeholder canvas + disabled construction tools (Lab-PR2)
  - Dev notes: run server + `npm run dev`
- **Done when:** empty shell loads; shows server health — **met** (`npm run build` green)

### Lab-PR2 — Site construction canvas (intake / penstock / turbine) ✅ done

- **Title:** `feat(lab): clean-slate site canvas — place intake, penstock, turbine`
- **Affects:** `src/lib/site.ts`, `SiteCanvas.vue`, `SelectionPanel.vue`, `App.vue`
- **Depends on:** Lab-PR1
- **Description:**
  - Empty site by default (`New`)
  - Tools: place **intake**, add **penstock bend**, place **turbine**, select/move/delete
  - x = ground distance (m), y = elevation (m); SVG canvas with drag
  - Minimum complete layout: intake + turbine; bends optional (sorted by s)
  - Contextual empty states and status line
- **Done when:** user can build a simple two- or three-point run on a blank grid — **met**

### Lab-PR3 — Compile site → plant + property panel + steady preview ✅ done

- **Title:** `feat(lab): compile site to plant JSON and engine preview`
- **Affects:** `compileSite.ts`, `plantParams.ts`, `PlantForm.vue`, `SteadyPreview.vue`, vitest
- **Depends on:** Lab-PR2
- **Description:**
  - Derive `grossHeadM`, `lengthM`, bend contribution to `minorLossCoefficient`
  - Plant-wide properties: stream, diameter, η, dynamics, operator inputs
  - Advanced overrides for friction/K/length/head
  - Steady preview via `createSession` + `set_hydro_input` (target kW / head)
  - Unit tests for compile math (`npm test` — 10 cases)
- **Done when:** higher intake → higher target power; extra bend → higher K — **met**

### Lab-PR4 — Named configs save/load/export + optional import ✅ done

- **Title:** `feat(lab): named configuration store and JSON import/export`
- **Affects:** `configStore.ts`, `labDocument.ts`, `ConfigToolbar.vue`
- **Depends on:** Lab-PR3
- **Description:**
  - Save / Save as / delete named configs (localStorage), including **site geometry**
  - Export plant JSON (engine schema) and full lab document
  - Import lab docs or bare plant JSON (reconstructs two-point profile)
  - Default home remains clean slate
- **Done when:** save/reload named site; export plant for CLI — **met**

### Lab-PR5 — Trial runner + charts

- **Title:** `feat(lab): session trials with ramps and series charts`
- **Affects:** trial runner, charts, WS live optional
- **Depends on:** Lab-PR3
- **Description:**
  - Start session from compiled plant; advance fixed duration
  - Chart electrical power, turbine speed vs `simTimeS`
  - Snapshot strip: generation, head breakdown, warnings
  - Commands mid-trial: set gate, stop (spin-down)
  - Prefer REST advance for interval demos; add WS live tick for continuous play
- **Done when:** open-gate trial shows S-curve spin-up; drought / extra-bend scenarios visibly differ

### Lab-PR6 — Station loads panel (optional, later)

- **Title:** `feat(lab): optional station grid and load toggles`
- **Affects:** lab UI, session document composition
- **Depends on:** Lab-PR5 and a clear need from prototyping
- **Description:**
  - Only if plant tuning needs brownout / load storytelling
  - Compose session JSON with grid; toggle loads during trial
- **Done when:** constrained generation + heavy load shows shortage/brownout in the strip

### Lab-PR7 — Polish + compare (optional stretch)

- **Title:** `feat(lab): trial comparison and scenario mutations`
- **Depends on:** Lab-PR5
- **Description:**
  - Overlay last two trial series
  - Mutations on the **current site** (“+10 m intake elevation”, “extra bend”, “half stream flow”)—not preset full plants as the main path
- **Done when:** designer can answer “what happens with more elevation delta?” quickly

### Lab-PR8 — Optional Tauri shell (later)

- **Title:** `feat(lab): Tauri desktop shell for hydro-config-lab`
- **Depends on:** Lab-PR4–PR5 stable
- **Description:**
  - Vue frontendDist + native open/save
  - Optional in-process session commands
- **Done when:** double-clickable local app runs trials

---

## Suggested implementation order

```text
Lab-PR1 (scaffold)
  → Lab-PR2 (clean-slate canvas)
  → Lab-PR3 (compile + properties + preview)
  → Lab-PR4 (save/export/import)
  → Lab-PR5 (trials)
  → Lab-PR6 (loads, only if needed)
  → Lab-PR7 (compare)
  → Lab-PR8 (Tauri, optional)
```

Minimum useful product: **PR1–PR5** (build site → compile → save → trial).

---

## Engine work required?

| Need | Required for lab MVP? |
| --- | --- |
| New physics | No |
| Profile fields in core | No (compile in UI; preserve site metadata in lab saves) |
| Server changes | Unlikely; CORS already permissive |
| Database | No |
| Game wiring | **Deferred** until prototyping settles the engine |

---

## Acceptance criteria (lab MVP)

1. New config opens a **blank site**; user places intake, penstock, turbine.  
2. Geometry compiles to plant fields; major properties are editable.  
3. Config can be named and saved (including site layout).  
4. Trials run on **energy-sim-server**, not browser-only math.  
5. Elevation / bends / drought produce visibly different outcomes.  
6. Export JSON works with `energy-sim hydro eval` and `session run`.  
7. Importing fixtures is optional, not required to start.

---

## Out of scope

- Replacing welcome simulator in-place  
- Atomic Adventures control-room integration (after prototyping)  
- Multiplayer / auth / hosted multi-tenant lab  
- Component catalog picker  
- Tests written only to inflate coverage  
