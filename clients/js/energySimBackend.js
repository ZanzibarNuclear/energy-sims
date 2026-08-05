/**
 * Shared EnergySim host adapter: one session contract for HTTP and WASM.
 *
 * Game alpha: createWasmBackend after loading energy-sim-wasm.
 * Lab / hosted: createHttpBackend → energy-sim-server.
 *
 * Operations (both transports):
 *   createSession, getSnapshot, start, stop, advance, tick,
 *   applyCommands, setLoad, setHydroInput, history, checkpoint, dispose
 *
 * @see docs/design.md (dual transport)
 * @see docs/api.md
 */

import { createEnergySimClient } from "./energySimClient.js";
import {
  presentGrid,
  presentHydro,
  presentSnapshot,
} from "./energySimPresent.js";

export { presentGrid, presentHydro, presentSnapshot };

/**
 * HTTP backend (lab today; hosted game later).
 *
 * @param {{ baseUrl?: string, fetchImpl?: typeof fetch }} [options]
 * @returns {import('./energySimBackend.types.js').EnergySimBackend}
 */
export function createHttpBackend(options = {}) {
  const client = createEnergySimClient(options);

  return {
    kind: "http",
    baseUrl: client.baseUrl,

    health() {
      return client.health();
    },

    async createSession(config) {
      const res = await client.createSession(config);
      return { sessionId: res.sessionId, snapshot: res.snapshot };
    },

    getSnapshot(sessionId) {
      return client.getSnapshot(sessionId);
    },

    start(sessionId) {
      return client.start(sessionId);
    },

    stop(sessionId) {
      return client.stop(sessionId);
    },

    advance(sessionId, { durationSecs, commands } = {}) {
      return client.advance(sessionId, {
        durationSecs,
        ...(commands?.length ? { commands } : {}),
      });
    },

    tick(sessionId, { dtSecs } = {}) {
      return client.tick(sessionId, { dtSecs });
    },

    applyCommands(sessionId, commands) {
      return client.applyCommands(sessionId, commands);
    },

    setLoad(sessionId, id, drawing) {
      return client.setLoad(sessionId, id, drawing);
    },

    setHydroInput(sessionId, fields) {
      return client.setHydroInput(sessionId, fields);
    },

    history(sessionId, opts = {}) {
      return client.history(sessionId, opts);
    },

    checkpoint(sessionId) {
      return client.checkpoint(sessionId);
    },

    /** No-op for HTTP (server owns session lifetime). */
    dispose(_sessionId) {
      return undefined;
    },

    /** Optional live WebSocket (HTTP only). */
    connectLive(sessionId, opts) {
      return client.connectLive(sessionId, opts);
    },
  };
}

/**
 * WASM backend (game alpha on the player device).
 *
 * @param {{
 *   Session: new (configJson: string) => WasmSessionHandle,
 *   version?: () => string,
 *   idFactory?: () => string,
 * }} wasm
 *   `Session` is the long-lived handle exported by energy-sim-wasm
 *   (`new Session(configJson)`, `Session.fromCheckpoint(json)`).
 * @returns {import('./energySimBackend.types.js').EnergySimBackend}
 */
export function createWasmBackend(wasm) {
  if (!wasm?.Session) {
    throw new Error("createWasmBackend requires wasm.Session (energy-sim-wasm)");
  }

  /** @type {Map<string, any>} */
  const sessions = new Map();
  let seq = 0;
  const nextId =
    wasm.idFactory ??
    (() => {
      seq += 1;
      return `wasm-${seq}-${Date.now().toString(36)}`;
    });

  function get(sessionId) {
    const s = sessions.get(sessionId);
    if (!s) {
      const err = new Error(`unknown session: ${sessionId}`);
      err.status = 404;
      throw err;
    }
    return s;
  }

  function configToJson(config) {
    return typeof config === "string" ? config : JSON.stringify(config);
  }

  return {
    kind: "wasm",

    health() {
      return Promise.resolve({
        ok: true,
        engine: typeof wasm.version === "function" ? wasm.version() : "energy-sim-wasm",
      });
    },

    async createSession(config) {
      const handle = new wasm.Session(configToJson(config));
      const sessionId = nextId();
      sessions.set(sessionId, handle);
      const snapshot = handle.snapshot();
      return { sessionId, snapshot };
    },

    async createSessionFromCheckpoint(checkpoint) {
      const json =
        typeof checkpoint === "string" ? checkpoint : JSON.stringify(checkpoint);
      if (typeof wasm.Session.fromCheckpoint !== "function") {
        throw new Error("wasm.Session.fromCheckpoint is not available");
      }
      const handle = wasm.Session.fromCheckpoint(json);
      const sessionId = nextId();
      sessions.set(sessionId, handle);
      return { sessionId, snapshot: handle.snapshot() };
    },

    async getSnapshot(sessionId) {
      return get(sessionId).snapshot();
    },

    async start(sessionId) {
      return get(sessionId).start();
    },

    async stop(sessionId) {
      return get(sessionId).stop();
    },

    async advance(sessionId, { durationSecs, commands } = {}) {
      const handle = get(sessionId);
      const commandsJson =
        commands?.length > 0 ? JSON.stringify(commands) : undefined;
      return handle.advance(durationSecs, commandsJson);
    },

    async tick(sessionId, { dtSecs } = {}) {
      return get(sessionId).tick(dtSecs);
    },

    async applyCommands(sessionId, commands) {
      return get(sessionId).applyCommands(JSON.stringify(commands ?? []));
    },

    async setLoad(sessionId, id, drawing) {
      return get(sessionId).applyCommands(
        JSON.stringify([{ type: "set_load", id, drawing: Boolean(drawing) }]),
      );
    },

    async setHydroInput(sessionId, fields) {
      return get(sessionId).applyCommands(
        JSON.stringify([{ type: "set_hydro_input", ...fields }]),
      );
    },

    async history(sessionId, { fromSecs, toSecs } = {}) {
      return get(sessionId).history(fromSecs ?? undefined, toSecs ?? undefined);
    },

    async checkpoint(sessionId) {
      return get(sessionId).checkpoint();
    },

    dispose(sessionId) {
      const handle = sessions.get(sessionId);
      if (handle) {
        sessions.delete(sessionId);
        if (typeof handle.free === "function") {
          handle.free();
        }
      }
    },
  };
}

/**
 * Pick a backend from options.
 *
 * @param {{
 *   kind?: 'http' | 'wasm',
 *   baseUrl?: string,
 *   fetchImpl?: typeof fetch,
 *   wasm?: { Session: Function, version?: Function },
 * }} [options]
 */
export function createEnergySimBackend(options = {}) {
  const kind =
    options.kind ??
    (options.wasm?.Session ? "wasm" : "http");

  if (kind === "wasm") {
    return createWasmBackend(options.wasm);
  }
  return createHttpBackend(options);
}
