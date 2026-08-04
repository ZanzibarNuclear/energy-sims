# energy-sim JS client

Minimal host client for browser apps. Today this package talks to **`energy-sim-server`** (REST + WebSocket). Game alpha will prefer **WASM** with the **same session operations**; treat this client as the HTTP transport and the shape of the future adapter.

Architecture: [docs/design.md](../../docs/design.md). Remaining work: [docs/plans/next.md](../../docs/plans/next.md).

## Usage (HTTP)

```js
import { createEnergySimClient, presentSnapshot } from "./energySimClient.js";

const client = createEnergySimClient({
  baseUrl: import.meta.env.VITE_ENERGY_SIM_URL ?? "http://127.0.0.1:8787",
});

const { sessionId, snapshot } = await client.createSession(stationConfig);
await client.start(sessionId);
await client.setLoad(sessionId, "lighting.main", true);
const report = await client.advance(sessionId, { durationSecs: 30 });
const view = presentSnapshot(report.snapshot);
// view.lightLevel → 1.0 normal, 0.4 brownout, 0 offline
```

Live channel:

```js
const live = client.connectLive(sessionId, {
  onMessage: (msg) => {
    if (msg.type === "snapshot") console.log(presentSnapshot(msg.snapshot));
  },
});
live.tick(1.0);
```

## Game integration notes

| Topic | Guidance |
| --- | --- |
| Short-term deploy | **WASM** on the player device (long-lived session API — see plan A1) |
| Hosted / lab path | HTTP client above + `VITE_ENERGY_SIM_URL` |
| Adapter | One interface for create/start/advance/command/snapshot; swap WASM vs HTTP |
| Console | Hydro sensors view + grid view from the same session |
| Brownout | Report-only; use `presentSnapshot(...).lightLevel` (or status) for dimming |
| Load ids | From station fixtures (`lighting.main`, `ev-charge.port-1`, …) — keep stable |
| Legacy hydro JS | Remove in the game repo once this path is default |

See [docs/api.md](../../docs/api.md) for HTTP/WS and WASM surfaces.
