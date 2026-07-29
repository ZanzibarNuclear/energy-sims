# energy-sim JS client

Minimal REST + WebSocket client for `energy-sim-server`. Intended for Atomic Adventures control-room wiring and other browser hosts.

## Usage

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

- Keep legacy `game/src/lib/simulations/hydro` prototypes until the remote path is the default.
- Opt in with `VITE_ENERGY_SIM_URL` (or equivalent).
- Brownout is **report-only** in the engine: use `presentSnapshot(...).lightLevel` (or `brownout`) to dim lights in the host.
- Load commands use engine load ids from station fixtures (`lighting.main`, `ev-charge.port-1`, …).

See [docs/api.md](../../docs/api.md) for the full HTTP/WS surface.
