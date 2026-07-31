# Engine follow-ups

**Status:** Active backlog after Stage 1 engine ship  
**Date:** 2026-07-30  
**Related design:** [../design.md](../design.md)

## Verification summary (Stage 1 PR stack)

Checked against workspace on 2026-07-30: `cargo test --workspace` (38 tests green), CLI smoke (eval / run / resume / status), server routes present.

| Item | Complete? | Tests / evidence | Disposition |
| --- | --- | --- | --- |
| PR1 Scaffold workspace | Yes | Builds | Closed — in design as layout |
| PR2 Hydro core evaluation | Yes | 15 core unit tests (equation, losses, config) | Closed |
| PR3 Session, ramps, history | Yes | Dynamics + session tests (spin-up/down, energy uses actual) | Closed |
| PR4 Station grid / brownout | Yes | Grid + session brownout tests; no auto-shed | Closed |
| PR5 CLI workflows | Yes | Manual smoke OK; **no automated CLI suite** | Closed functionally; test gap below |
| PR6 Persistence packages | Yes | Checkpoint RT, mid-ramp package, atomic write | Closed |
| PR7 REST + WebSocket server | Yes (code) | **0 automated server tests** | Closed functionally; test gap below |
| PR8 Game control-room client | Partial | `clients/js/` client + brownout presentation helper; **not wired into Atomic Adventures** | Open — this plan |
| PR9 Optional WASM | Yes (minimal) | 2 unit tests (version, evaluate) | Closed for optional path |

Items below are **not** incomplete Stage 1 physics; they are the next product and quality work.

---

## Prioritized short list (engine / product completeness)

The Stage 1 base is solid. Recommended order:

| Priority | Item | Why now / why later |
| --- | --- | --- |
| **P0** | **Hydro config lab UI** | Highest leverage for tuning plants and holo-reader trial design. Clean-slate construction against the production engine. Plan: [hydro-config-lab.md](hydro-config-lab.md). |
| **P1** | **Game control-room client (PR8 remainder)** | **After** config prototyping settles the engine and clears latent issues. Do not parallel aggressively. |
| **P2** | **Targeted automated tests** | Only where they prevent real regressions (e.g. lab compile math, flaky host paths that actually break). Not a focus during early lab UI. |
| **P3** | **Database session store** | Files + in-memory server sessions are enough for single-author lab and local game alpha. **Defer.** |
| Later | Component catalog resolver | Sponsorship / shop UX; optional `packageId` hooks already allowed |
| Later | Multi-source plants (solar, battery, fission, fusion) | Same bus model; only after hydro + lab + game path feel good |
| Later | Auto load-shed policies | Gameplay polish beyond report-only brownout |

**Conclusion:** Nothing more pressing on the engine core than **using it via the config lab**. Game wiring and database wait. Tests only when they protect real regressions.

---

## Work items

### 1. Hydro config lab (see dedicated plan)

- Design: [../hydro-config-lab.md](../hydro-config-lab.md)
- Plan: [hydro-config-lab.md](hydro-config-lab.md)
- Outcome: iterate plant geometry and operator conditions against production sessions; export fixtures for the game and lesson experiments.

### 2. Integration test hardening

**Goal:** CI-proof CLI and server without slowing unit tests.

Suggested additions (small PRs):

- **CLI smoke (Rust or shell):** `session run` fixture → assert files exist, CSV has expected headers, `sim_time_s` advanced, resume continues.
- **Server HTTP tests:** axum `oneshot` or `TestClient` for create → start → advance → snapshot; brownout under heavy load command sequence.
- **Optional WS test:** one tick round-trip on `/live`.

### 3. Game control-room client (remainder of PR8)

**Affects:** `atomic-adventures` (primary), optionally polish `clients/js/`.

- Point console at `VITE_ENERGY_SIM_URL` / config.
- Subscribe to WS live or poll snapshot for generation, load, margin, brownout.
- Send load on/off and hydro operator commands through REST/WS.
- Present brownout (e.g. dim lights via `presentSnapshot().lightLevel`).
- Leave legacy in-game hydro prototypes until the remote path is default.

### 4. Database session store (deferred)

When needed (multi-session server, shared multiplayer, long-running remote ops):

- Keep JSON checkpoint as interchange / export format.
- Add store behind the server (SQLite for local; Postgres if multi-tenant).
- Map: session id → config, phase, sim time, optional blob checkpoint, series/events partitions or object storage.
- Do **not** block the config lab or game alpha on this.

### 5. Component catalog resolver (later)

- Resolve `packageId` → generic field set.
- Catalog JSON/YAML under fixtures or a future `catalog/` crate.
- UI pickers in lab and shop; physics still reads expanded fields only.

### 6. Multi-source plants (later)

- Formalize `Source` attachment on the bus (hydro is the first).
- Solar / battery / fission / fusion implement available power + ramps as needed.
- Session config gains a sources array; console shows multi-source mix.

### 7. Auto load-shed (later)

- Policy beyond report-only brownout (priority tiers, deferrable EV, etc.).
- Host may still present drama; engine can optionally turn loads off under policy.

---

## Out of scope here

- Changing Stage 1 physics equations without a separate design revision.
- Replacing the welcome `HydroPowerSimulator.vue` one-shot — the lab supersedes that role for production-engine work.
