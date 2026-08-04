# Next work: game-ready hydro + grid

**Status:** Active  
**Date:** 2026-08-04  
**Design:** [../design.md](../design.md) (integration, Clearwater workflow, dual transport)  
**Lab design:** [../hydro-config-lab.md](../hydro-config-lab.md) (MVP shipped)

This is the **single** implementation plan for remaining work. Stage 1 engine and hydro config lab MVP are closed (recorded in design history).

---

## Intent

Make energy-sims the **calculation backend** for Atomic Adventures Part I:

1. **Hydro + station grid** for the control-room terminals (sensors + bus/loads).  
2. **Same engine** for holo-reader hydro lessons (reduced knobs, fixture scenarios).  
3. **Clearwater** plant of record maintained via lab → fixture → game workflow.  
4. **WASM on the player device** for short-term deploy; **remote server** when we host shared logic.

**Sequencing:** finish backend / host surface in this repo → plug into game integration points → remove legacy hydro prototypes in the game repo.

---

## Guiding decisions (do not reopen lightly)

| Decision | Choice |
| --- | --- |
| Short-term game deploy | **WASM** (player device) |
| Long-term shared logic | **energy-sim-server** (REST + WebSocket) |
| API shape | One **session contract**; transport-swappable adapter |
| Control console | **Two views**, one session: hydro sensors + station grid |
| Brownout | Report-only until a later shed design |
| Lab desktop shell | **No Tauri** |
| Physics authority | energy-sims only; delete game JS hydro when adapter is default |

---

## Workstreams

### A. Host surface (this repo) — do first

#### A1. Long-lived WASM session API

**Why:** Current `energy-sim-wasm` is one-shot (`evaluateHydro`, `runSession`, `sessionSnapshot`). Control room and holo trials need create → many advance/command/snapshot calls without losing state.

**Affects:** `crates/energy-sim-wasm/`

**Done when:**

- JS can create a session from plant or station JSON, hold a handle, `start` / `stop`, `advance` / `tick`, apply `Command`s, read `Snapshot`, read history samples/events, free the handle.
- Checkpoint serialize/restore optional but desirable for save tests.
- Unit or wasm-bindgen tests cover spin-up + load toggle + brownout status on a station fixture.

#### A2. Grid presentation for consoles

**Why:** Aggregate margin/status exists; a grid terminal needs **per-load** rows and stable ids for UI and game circuits.

**Affects:** `energy-sim-runtime` snapshot (or adjacent DTO), WASM + HTTP responses, [station-grid.md](../station-grid.md)

**Done when:**

- Snapshot (or documented companion payload) includes load list: `id`, `label`, `ratingW`, `priority`, `drawing`.
- Station fixture smoke shows EV on → shortage/brownout with load table reflecting state.
- No second physics model—presentation only.

#### A3. Shared host client / adapter sketch

**Why:** Game and lab should not invent divergent APIs.

**Affects:** `clients/js/` (and types usable from the game), docs in [api.md](../api.md)

**Done when:**

- Documented `EnergySimBackend` operations match WASM and HTTP.
- Thin JS helpers: create backend, present snapshot for hydro strip + grid strip (e.g. `lightLevel` from brownout), apply load/hydro commands.
- README spells out game alpha = WASM; server = lab today and hosted game later.

#### A4. Clearwater fixture workflow (process + light tooling)

**Why:** Lab export already exists; change needs a repeatable promote path.

**Affects:** `fixtures/`, `fixtures/README.md`, optional script under `scripts/` or `examples/`

**Done when:**

- Documented steps: lab export → CLI smoke → PR plant + station → note version/changelog.
- Station document remains plant + grid; load ids stable.
- Optional: one script that runs `hydro eval` + short `session run` on Clearwater fixtures for CI or local check.
- Game consumption rule: load fixture JSON through the adapter, no forked constants.

---

### B. Lab follow-ons (this repo) — as needed for design

Not blockers for WASM host work; useful for authoring and holo research.

| ID | Item | Done when |
| --- | --- | --- |
| B1 | **Station loads in lab** | Session composed with grid; toggle loads mid-trial; margin/brownout visible |
| B2 | **Compare / mutations** | Overlay recent series or one-click “+ head / bend / drought” on current site |
| B3 | **WS live trials** | Optional; parity with hosted console feel |

Prefer B1 before deep control-room UX work if designers need brownout intuition without the game.

---

### C. Atomic Adventures integration (game repo) — after A

Do not parallel aggressively until A1–A3 are usable.

#### C1. Wire EnergySim adapter

- Bundle or load WASM artifact (or point at server in dev).
- Create station session from Clearwater fixture on facility start / control-room open (exact lifecycle with game time is a game design choice; engine only advances when asked).
- Map game load circuits ↔ engine load ids.

#### C2. Control room terminals

- **Hydro sensors** view: snapshot fields + charts from history/samples.  
- **Grid** view: bus + load table + commands.  
- Presentation only; no local power equation.

#### C3. Holo-reader hydro module

- Use reduced knobs proven in the lab.  
- Scenario JSON from fixtures or lesson-embedded configs.  
- Short trials via same adapter (WASM).  
- Keep quiz/pages; add simulator-backed steps only where they teach.

#### C4. Remove legacy hydro prototypes

- Delete or quarantine `game/src/lib/simulations/hydro/` (and any duplicate math).  
- Update contracts: `hydro-simulator.md`, `station-electrical-grid.md`, `control-panel.md` to name energy-sims as authority.  
- Leave welcome monorepo sim alone unless desired for marketing demos.

---

### D. Quality and deferred (as needed)

| Item | Priority |
| --- | --- |
| Automated CLI smoke (run → files → headers) | P2 when CI flakes or regressions bite |
| Automated server HTTP tests | P2 same |
| Database session store | Deferred (multi-session hosted product) |
| Component catalog resolver | Later |
| Multi-source plants | Later (after hydro+grid path feels good) |
| Auto load-shed policies | Later |

---

## Suggested order

```text
A1 WASM long-lived session
  → A2 grid load table on snapshot
  → A3 host client / adapter docs + helpers
  → A4 fixture workflow hardening
  → (optional B1 lab loads for designers)
  → C1–C3 game wiring
  → C4 remove legacy hydro
```

Minimum useful product for game plug-in: **A1 + A2 + A3** with existing Clearwater station fixture.

---

## Acceptance criteria (backend-ready for game)

1. A browser host can run a **station** session entirely in WASM: start, advance, open gate, toggle EV load, observe brownout/margin, chart series.  
2. Snapshot (or documented DTO) supports **hydro terminal** and **grid terminal** without inventing numbers in the game.  
3. Clearwater station fixture is the documented plant of record; promote path from lab export is written and smoke-checked.  
4. HTTP/server path still works for the lab (no regression).  
5. Design docs name dual transport and integration boundaries (done in design rev 10; keep in sync when APIs change).

---

## Out of scope

- Tauri / Electron lab packaging  
- Replacing Stage 1 physics equations without a design revision  
- Multiplayer authoritative sim server (future; contract already dual-transport)  
- Full component sponsorship catalog  
- Rewriting welcome `HydroPowerSimulator.vue` in place  

---

## Closed work (do not re-plan)

| Area | Where recorded |
| --- | --- |
| Stage 1 engine PR stack | [design.md](../design.md) implementation history |
| Hydro config lab MVP + in-flight UX | [hydro-config-lab.md](../hydro-config-lab.md), design history |
