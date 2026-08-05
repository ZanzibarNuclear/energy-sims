# energy-sim JS host clients

Browser/Node helpers for Atomic Adventures, the hydro config lab, and other hosts.

| Module | Role |
| --- | --- |
| [`energySimBackend.js`](energySimBackend.js) | **Preferred** — transport-swappable adapter (`http` \| `wasm`) |
| [`energySimPresent.js`](energySimPresent.js) | Hydro + grid presentation strips from a `Snapshot` |
| [`energySimClient.js`](energySimClient.js) | Low-level REST + WebSocket client for `energy-sim-server` |

Architecture: [docs/design.md](../../docs/design.md). API: [docs/api.md](../../docs/api.md). Plan: [docs/plans/next.md](../../docs/plans/next.md).

## Deploy paths

| Host | Backend | Notes |
| --- | --- | --- |
| **Game alpha** | **WASM** on the player device | `createWasmBackend` after `wasm-pack` / bundler load of `energy-sim-wasm` |
| **Hydro config lab** | **HTTP** → local `energy-sim-server` | `createHttpBackend` / existing lab client |
| **Hosted game (later)** | **HTTP** → shared server | Same adapter; swap `kind` / URL |

Do not reimplement hydro or bus math in the game. Load **Clearwater Station** (e.g. `fixtures/stations/clearwater-station.json`, which embeds **Clearwater Diversion**) through the adapter.

## Adapter usage

### HTTP (lab / server)

```js
import {
  createHttpBackend,
  presentSnapshot,
  presentHydro,
  presentGrid,
} from "./energySimBackend.js";

const backend = createHttpBackend({
  baseUrl: import.meta.env.VITE_ENERGY_SIM_URL ?? "http://127.0.0.1:8787",
});

const { sessionId, snapshot } = await backend.createSession(stationConfig);
await backend.start(sessionId);
await backend.setLoad(sessionId, "lighting.main", true);
const report = await backend.advance(sessionId, { durationSecs: 30 });
const view = presentSnapshot(report.snapshot);
// view.hydro — sensors terminal
// view.grid  — bus + loads[] + lightLevel
```

### WASM (game alpha)

```js
import init, { Session, version } from "energy-sim-wasm"; // path depends on pack
import { createWasmBackend, presentGrid } from "./energySimBackend.js";

await init();
const backend = createWasmBackend({ Session, version });

const { sessionId } = await backend.createSession(stationConfig);
await backend.start(sessionId);
await backend.advance(sessionId, { durationSecs: 20 });
await backend.setLoad(sessionId, "ev-charge.port-1", true);
const snap = await backend.getSnapshot(sessionId);
const grid = presentGrid(snap); // loads table + brownout lightLevel
backend.dispose(sessionId); // free WASM handle when leaving console
```

Or:

```js
import { createEnergySimBackend } from "./energySimBackend.js";
const backend = createEnergySimBackend({ kind: "wasm", wasm: { Session, version } });
// or
const backend = createEnergySimBackend({ kind: "http", baseUrl: "http://127.0.0.1:8787" });
```

## Session contract

| Operation | Meaning |
| --- | --- |
| `createSession(config)` | Plant or station JSON → `{ sessionId, snapshot }` |
| `start` / `stop` | Session phase |
| `advance` / `tick` | Sim time |
| `applyCommands` / `setLoad` / `setHydroInput` | Operator + loads |
| `getSnapshot` / `history` | Console + charts |
| `checkpoint` | Serialize for save / transfer |
| `dispose` | Drop WASM handle (HTTP no-op) |

## Presentation helpers

| Helper | Use |
| --- | --- |
| `presentHydro(snapshot)` | Control-room hydro sensors terminal |
| `presentGrid(snapshot)` | Bus, margin, status, `loads[]`, `lightLevel` |
| `presentSnapshot(snapshot)` | Combined object (includes nested `hydro` / `grid`) |

Brownout is **report-only** in the engine: dim lights from `lightLevel` or `gridStatus`. Load ids come from station fixtures (`lighting.main`, `ev-charge.port-1`, …) and must stay stable once the game binds circuits.

## Legacy game hydro

Keep in-game JS hydro prototypes until this adapter path is the default, then remove them in the game repo.
