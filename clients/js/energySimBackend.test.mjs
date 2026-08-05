/**
 * Adapter smoke tests with a mock WASM Session.
 * Run: node --test clients/js/energySimBackend.test.mjs
 */
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { createWasmBackend } from "./energySimBackend.js";

function mockSnapshot(overrides = {}) {
  return {
    simTimeS: 0,
    electricalPowerKw: 0,
    gridStatus: "ok",
    loads: [],
    ...overrides,
  };
}

class MockSession {
  constructor(configJson) {
    this._config = configJson;
    this._t = 0;
    this._phase = "configured";
    this._loads = new Map();
  }

  static fromCheckpoint(json) {
    const s = new MockSession("{}");
    const doc = JSON.parse(json);
    s._t = doc.simTimeS ?? 0;
    s._phase = "running";
    return s;
  }

  start() {
    this._phase = "running";
    return mockSnapshot({ simTimeS: this._t, phase: this._phase });
  }

  stop() {
    this._phase = "stopped";
    return mockSnapshot({ simTimeS: this._t, phase: this._phase });
  }

  advance(durationSecs, commandsJson) {
    if (commandsJson) {
      const cmds = JSON.parse(commandsJson);
      for (const c of cmds) {
        if (c.type === "set_load") this._loads.set(c.id, c.drawing);
      }
    }
    this._t += durationSecs;
    return {
      snapshot: mockSnapshot({
        simTimeS: this._t,
        electricalPowerKw: 5,
        loads: [...this._loads.entries()].map(([id, drawing]) => ({
          id,
          ratingW: 100,
          drawing,
        })),
      }),
      energyIntervalKwh: 0.01,
      samplesAdded: 1,
      eventsAdded: 1,
    };
  }

  tick(dt) {
    this._t += dt;
    return mockSnapshot({ simTimeS: this._t });
  }

  applyCommands(json) {
    const cmds = JSON.parse(json);
    for (const c of cmds) {
      if (c.type === "set_load") this._loads.set(c.id, c.drawing);
    }
    return mockSnapshot({
      simTimeS: this._t,
      loads: [...this._loads.entries()].map(([id, drawing]) => ({
        id,
        ratingW: 3500,
        drawing,
      })),
    });
  }

  snapshot() {
    return mockSnapshot({ simTimeS: this._t, phase: this._phase });
  }

  history() {
    return { events: [], samples: [] };
  }

  checkpoint() {
    return { kind: "energy-sim-checkpoint", simTimeS: this._t };
  }

  free() {
    this._freed = true;
  }
}

describe("createWasmBackend", () => {
  it("runs create → start → advance → setLoad → dispose", async () => {
    const backend = createWasmBackend({
      Session: MockSession,
      version: () => "mock-wasm",
      idFactory: () => "s1",
    });

    const health = await backend.health();
    assert.equal(health.ok, true);

    const { sessionId, snapshot } = await backend.createSession({ kind: "hydro-plant" });
    assert.equal(sessionId, "s1");
    assert.ok(snapshot);

    await backend.start(sessionId);
    const report = await backend.advance(sessionId, {
      durationSecs: 10,
      commands: [{ type: "set_load", id: "ev-charge.port-1", drawing: true }],
    });
    assert.equal(report.snapshot.simTimeS, 10);

    const after = await backend.setLoad(sessionId, "lighting.main", true);
    assert.equal(after.loads.length, 2);

    const ck = await backend.checkpoint(sessionId);
    assert.equal(ck.kind, "energy-sim-checkpoint");

    backend.dispose(sessionId);
    await assert.rejects(() => backend.getSnapshot(sessionId));
  });
});
