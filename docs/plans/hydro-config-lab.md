# Plan: Hydro Config Lab

**Status:** Ready to implement  
**Date:** 2026-07-30  
**Design:** [../hydro-config-lab.md](../hydro-config-lab.md)  
**Depends on:** Stage 1 engine (shipped) — server + plant/session JSON  
**Priority:** **Next product build** (ahead of database and catalog)

---

## Intent

Ship a Vue-based **config + trial** tool that authors plant geometry and parameters, runs them through `energy-sim-server`, and exports fixtures for the game and lesson experiments.

Stack for MVP:

- **Vue 3 + Vite + TypeScript**
- **energy-sim-server** on localhost (REST + WebSocket)
- Local named configs + JSON import/export
- **No Tauri / Electron required** for first usable version (Tauri shell is a later optional PR)

---

## PR breakdown

### Lab-PR1 — Scaffold lab app

- **Title:** `feat(lab): scaffold Vue Vite hydro-config-lab`
- **Affects:** `apps/hydro-config-lab/` (or `clients/hydro-config-lab/`), root README pointer, lab README
- **Depends on:** none (engine already exists)
- **Description:**
  - Vite + Vue 3 + TS app
  - Env `VITE_ENERGY_SIM_URL` default `http://127.0.0.1:8787`
  - Health check banner (server reachable / not)
  - Port or copy `clients/js` client into typed module
  - Dev script notes: run server + `npm run dev`
- **Done when:** empty shell loads; shows server health

### Lab-PR2 — Plant form + steady preview

- **Title:** `feat(lab): plant property form and live evaluate preview`
- **Affects:** lab UI, client create-session or eval path
- **Depends on:** Lab-PR1
- **Description:**
  - Forms for stream, penstock scalars, turbine, generator, dynamics, operator inputs
  - Load defaults from `fixtures/plants/clearwater-diversion.json` and `ideal-teaching.json`
  - Steady preview: create session or evaluate → show \(P_e\), head breakdown, warnings
  - Name field; serialize/deserialize plant JSON
- **Done when:** editing diameter/flow/head updates preview via production engine

### Lab-PR3 — Profile editor (x–y grid)

- **Title:** `feat(lab): penstock profile editor compiles to plant geometry`
- **Affects:** `ProfileEditor.vue`, `compileProfile.ts`, unit tests for compiler
- **Depends on:** Lab-PR2
- **Description:**
  - Interactive polyline: x = ground distance (m), y = elevation (m)
  - Derive `grossHeadM`, `lengthM`, bend contribution to `minorLossCoefficient`
  - Advanced overrides for friction/K/length/head
  - Preserve profile points in lab config metadata so save/reload restores the grid
  - Seed profile that matches Clearwater head/length approximately
- **Done when:** dragging intake higher increases preview power; extra bend increases loss / lowers power for same diameter and flow

### Lab-PR4 — Named configs save/load/export

- **Title:** `feat(lab): named configuration store and JSON import/export`
- **Affects:** config store, export UI
- **Depends on:** Lab-PR2 (can parallel Lab-PR3 after form exists)
- **Description:**
  - Save / Save as / delete named configs (localStorage)
  - Download plant JSON and full session document (plant + optional grid)
  - Import JSON files
  - Document how to copy into `fixtures/plants/` for the repo
- **Done when:** author can close browser, reopen, and continue a named config; export validates in CLI `hydro eval`

### Lab-PR5 — Trial runner + charts

- **Title:** `feat(lab): session trials with ramps and series charts`
- **Affects:** trial runner, charts, WS live optional
- **Depends on:** Lab-PR2
- **Description:**
  - Start session from current config; advance fixed duration (e.g. 30 / 120 s)
  - Chart electrical power, turbine speed, margin vs `simTimeS`
  - Snapshot strip: generation, load, margin, gridStatus
  - Commands mid-trial: set gate, set load drawing, stop (spin-down)
  - Prefer REST advance for interval demos; add WS live tick for continuous play
- **Done when:** open-gate trial shows S-curve spin-up matching engine ramp defaults; drought / heavy load scenarios visible

### Lab-PR6 — Station loads panel

- **Title:** `feat(lab): optional station grid and load toggles in trials`
- **Affects:** lab UI, session document composition
- **Depends on:** Lab-PR5
- **Description:**
  - Compose session JSON with grid from `fixtures/grids/utility-station.json` or editable load list
  - Toggle loads during trial; show brownout presentation
  - Scenario presets: “lights only”, “+ EV”, “kitchen overload”
- **Done when:** turning on EV charger with constrained generation produces shortage/brownout in the strip

### Lab-PR7 — Polish + compare (optional stretch)

- **Title:** `feat(lab): trial comparison and scenario presets`
- **Depends on:** Lab-PR5
- **Description:**
  - Side-by-side or overlay last two trial series
  - Preset buttons: “+10 m head”, “extra bend”, “half stream flow”
  - README scenarios for holo-reader research notes
- **Done when:** designer can answer “what happens with more elevation delta?” in under a minute without hand-editing JSON

### Lab-PR8 — Optional Tauri shell (later)

- **Title:** `feat(lab): Tauri desktop shell for hydro-config-lab`
- **Depends on:** Lab-PR4–PR5 stable
- **Description:**
  - `create-tauri-app` / add `src-tauri` with Vue frontendDist
  - Native open/save for config JSON
  - Optional: Tauri commands wrapping `Session` in-process (may still keep HTTP path for parity with game)
- **Done when:** double-clickable local app runs trials without a separate browser window (server may still auto-start or in-process)

---

## Suggested implementation order

```text
Lab-PR1 → Lab-PR2 → Lab-PR3 ─┐
                └→ Lab-PR4 ──┼→ Lab-PR5 → Lab-PR6 → Lab-PR7
                             │
                    Lab-PR8 (optional, after useful web MVP)
```

Minimum useful product: **PR1–PR5** (form + profile + save + trials).  
Grid/brownout **PR6** is highly recommended before declaring the lab “complete” for game tuning.

---

## Engine work required?

| Need | Required for lab MVP? |
| --- | --- |
| New physics | No |
| Profile fields in core | No (compile in UI; optional metadata) |
| Server changes | Unlikely; use existing API |
| Server/CLI automated tests | Nice parallel (see [engine-followups.md](engine-followups.md)) — not a blocker |
| Database | No |

If live WS or CORS becomes awkward for Vite dev origin, add a small server CORS allowlist or Vite proxy—implementation detail during Lab-PR1/PR5.

---

## Acceptance criteria (lab MVP)

1. Author can define a penstock on an x–y elevation grid and see derived head/length.  
2. All major plant properties are editable; config can be named and saved.  
3. Submitting a config runs on **energy-sim-server** (or equivalent production engine path), not browser-only math.  
4. A trial shows ramp behavior and final power; drought and bend changes produce visibly different outcomes.  
5. Export JSON works with `energy-sim hydro eval` and `session run`.  
6. Clearwater-like and ideal-teaching starting points are one click away.

---

## Out of scope

- Replacing welcome simulator in-place  
- Full Atomic Adventures control-room integration (tracked under engine follow-ups PR8 remainder)  
- Multiplayer / auth / hosted multi-tenant lab  
- Component catalog picker (later engine follow-up)
