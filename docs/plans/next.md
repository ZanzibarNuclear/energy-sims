# Next work: game-ready hydro + grid

**Status:** Host surface complete — game integration next  
**Updated:** 2026-08-05  
**Design:** [../design.md](../design.md) (integration, Clearwater workflow, dual transport)  
**Lab design:** [../hydro-config-lab.md](../hydro-config-lab.md) (MVP shipped)

Stage 1 engine, hydro config lab MVP, and **host surface (A1–A4)** are closed. This plan is the handoff surface: what energy-sims delivers for Atomic Adventures, and what remains in the **game repo**.

---

## Intent

Make energy-sims the **calculation backend** for Atomic Adventures Part I:

1. **Hydro + station grid** for the control-room terminals (sensors + bus/loads).  
2. **Same engine** for holo-reader hydro lessons (reduced knobs, fixture scenarios).  
3. **Clearwater** plant of record maintained via lab → fixture → game workflow.  
4. **WASM on the player device** for short-term deploy; **remote server** when we host shared logic.

**Sequencing:** ~~finish backend / host surface in this repo~~ → **plug into game integration points** → remove legacy hydro prototypes in the game repo.

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

### Clearwater naming

| Name | Role |
| --- | --- |
| **Clearwater Run** | Stream (story) |
| **Clearwater Diversion** | Hydro plant — `fixtures/plants/clearwater-diversion.json` |
| **Clearwater Station** | Session (plant + bus) — `fixtures/stations/clearwater-station.json` |

---

## Workstreams

### A. Host surface (this repo) — ✅ complete

Minimum useful product for game plug-in: **A1 + A2 + A3** + Clearwater station fixture — **met** (A4 workflow also shipped).

#### A1. Long-lived WASM session API ✅ done

**Shipped:** `WasmSession` (`Session` in JS) with start/stop/advance/tick/commands/snapshot/history/checkpoint; runtime `from_checkpoint_json` + `history_window`; tests on station fixture.

#### A2. Grid presentation for consoles ✅ done

**Shipped:** `Snapshot.loads: LoadSnapshot[]` (`id`, `label`, `ratingW`, `priority`, `drawing`); HTTP/WASM return the same shape; docs in [station-grid.md](../station-grid.md).

#### A3. Shared host client / adapter sketch ✅ done

**Shipped:** `clients/js/energySimBackend.js` (`createHttpBackend` / `createWasmBackend`), `energySimPresent.js` (`presentHydro` / `presentGrid` / `presentSnapshot`), README deploy paths.

#### A4. Clearwater fixture workflow (process + light tooling) ✅ done

**Shipped:** [fixtures/README.md](../../fixtures/README.md) promote steps, [fixtures/CHANGELOG.md](../../fixtures/CHANGELOG.md), `./scripts/smoke-clearwater.sh` (eval + session run). Naming: Diversion plant vs Station session/grid.

---

### B. Lab follow-ons (this repo) — optional / not blocking game

Useful for authoring and holo research. **Do not block** control-room or holo wiring in the game.

| ID | Item | Done when | Status |
| --- | --- | --- | --- |
| B1 | **Station loads in lab** | Session composed with grid; toggle loads mid-trial; margin/brownout visible | Open (nice-to-have) |
| B2 | **Compare / mutations** | Overlay recent series or one-click “+ head / bend / drought” on current site | Open |
| B3 | **WS live trials** | Optional; parity with hosted console feel | Open |

Prefer B1 only if designers need brownout intuition in the lab before the game console exists.

---

### C. Atomic Adventures integration (game repo) — **current focus**

Host surface in energy-sims is ready. Work below lives primarily in the **game** (and monorepo contracts).

#### C1. Wire EnergySim adapter

- Bundle or load WASM artifact (or point at server in dev).
- Create station session from **Clearwater Station** fixture on facility start / control-room open (lifecycle vs game time is a game design choice; engine only advances when asked).
- Map game load circuits ↔ engine load ids (`lighting.main`, `holo-reader.library`, `ev-charge.port-1`, `kitchen.appliance`).

#### C2. Control room terminals

- **Hydro sensors** view: snapshot fields + charts from history/samples.  
- **Grid** view: bus + load table + commands.  
- Presentation only; no local power equation. Use `presentHydro` / `presentGrid` (or equivalent) over raw snapshots.

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
A1–A4 host surface ✅
  → C1–C3 game wiring   ← you are here
  → C4 remove legacy hydro
  → (optional B1 lab loads if designers need it)
```

---

## Acceptance criteria (backend-ready for game)

| # | Criterion | Status |
| --- | --- | --- |
| 1 | Browser host can run a **station** session in WASM: start, advance, open gate, toggle EV load, observe brownout/margin, chart series | ✅ Met (`WasmSession` + station fixture tests) |
| 2 | Snapshot (or documented DTO) supports **hydro terminal** and **grid terminal** without inventing numbers in the game | ✅ Met (`Snapshot.loads`, presenters) |
| 3 | Clearwater station fixture is the documented plant of record; promote path from lab export is written and smoke-checked | ✅ Met (`clearwater-station` + `smoke-clearwater.sh`) |
| 4 | HTTP/server path still works for the lab (no regression) | ✅ Met (lab + `createHttpBackend`) |
| 5 | Design docs name dual transport and integration boundaries | ✅ Met (design rev 10+; keep in sync when APIs change) |

**energy-sims is backend-ready for game integration.** Remaining product work for Part I is almost entirely on the game side (stream C).

---

## Handoff checklist (game integration)

Use this when opening the game workstream:

| Asset | Where |
| --- | --- |
| Station config (plant + loads) | `fixtures/stations/clearwater-station.json` |
| Plant-only (eval / lessons) | `fixtures/plants/clearwater-diversion.json` |
| Stable load ids | `lighting.main`, `holo-reader.library`, `ev-charge.port-1`, `kitchen.appliance` |
| WASM package | `crates/energy-sim-wasm` (`wasm-pack build … --target web`) |
| JS adapter | `clients/js/energySimBackend.js`, `energySimPresent.js` |
| Session contract / API | [api.md](../api.md), [clients/js/README.md](../../clients/js/README.md) |
| Grid semantics | [station-grid.md](../station-grid.md) |
| Promote / smoke | [fixtures/README.md](../../fixtures/README.md), `./scripts/smoke-clearwater.sh` |

Nothing further in this repo is **required** before starting C1. Optional polish (B1, CI smoke in D) can wait until a concrete pain appears.

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
| Host surface A1–A4 | This plan (above) |
