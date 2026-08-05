/**
 * Minimal browser / Node client for energy-sim-server (REST + optional WebSocket).
 *
 * Usage:
 *   const client = createEnergySimClient({ baseUrl: 'http://127.0.0.1:8787' });
 *   const { sessionId, snapshot } = await client.createSession(configJson);
 *   await client.start(sessionId);
 *   const report = await client.advance(sessionId, { durationSecs: 60 });
 */

/**
 * @param {{ baseUrl?: string, fetchImpl?: typeof fetch }} [options]
 */
export function createEnergySimClient(options = {}) {
  const baseUrl = (options.baseUrl ?? "http://127.0.0.1:8787").replace(/\/$/, "");
  const fetchImpl = options.fetchImpl ?? globalThis.fetch;
  if (typeof fetchImpl !== "function") {
    throw new Error("energySimClient requires fetch (browser or Node 18+)");
  }

  async function request(path, init = {}) {
    const res = await fetchImpl(`${baseUrl}${path}`, {
      ...init,
      headers: {
        Accept: "application/json",
        ...(init.body ? { "Content-Type": "application/json" } : {}),
        ...init.headers,
      },
    });
    const text = await res.text();
    let body = null;
    if (text) {
      try {
        body = JSON.parse(text);
      } catch {
        body = text;
      }
    }
    if (!res.ok) {
      const msg = body?.error ?? res.statusText ?? `HTTP ${res.status}`;
      const err = new Error(msg);
      err.status = res.status;
      err.body = body;
      throw err;
    }
    return body;
  }

  return {
    baseUrl,

    health() {
      return request("/health");
    },

    /**
     * @param {string|object} config plant or energy-session JSON
     */
    createSession(config) {
      const body = typeof config === "string" ? config : JSON.stringify(config);
      return request("/v1/sessions", { method: "POST", body });
    },

    getSnapshot(sessionId) {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}`);
    },

    start(sessionId) {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}/start`, {
        method: "POST",
      });
    },

    stop(sessionId) {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}/stop`, {
        method: "POST",
      });
    },

    /**
     * @param {string} sessionId
     * @param {{ durationSecs: number, commands?: object[] }} body
     */
    advance(sessionId, body) {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}/advance`, {
        method: "POST",
        body: JSON.stringify(body),
      });
    },

    /**
     * @param {string} sessionId
     * @param {{ dtSecs: number }} body
     */
    tick(sessionId, body) {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}/tick`, {
        method: "POST",
        body: JSON.stringify(body),
      });
    },

    /**
     * @param {string} sessionId
     * @param {object[]} commands
     */
    applyCommands(sessionId, commands) {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}/commands`, {
        method: "POST",
        body: JSON.stringify({ commands }),
      });
    },

    setLoad(sessionId, id, drawing) {
      return this.applyCommands(sessionId, [
        { type: "set_load", id, drawing: Boolean(drawing) },
      ]);
    },

    setHydroInput(sessionId, fields) {
      return this.applyCommands(sessionId, [
        { type: "set_hydro_input", ...fields },
      ]);
    },

    history(sessionId, { fromSecs, toSecs } = {}) {
      const q = new URLSearchParams();
      if (fromSecs != null) q.set("fromSecs", String(fromSecs));
      if (toSecs != null) q.set("toSecs", String(toSecs));
      const qs = q.toString();
      return request(
        `/v1/sessions/${encodeURIComponent(sessionId)}/history${qs ? `?${qs}` : ""}`,
      );
    },

    checkpoint(sessionId) {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}/checkpoint`, {
        method: "POST",
      });
    },

    /**
     * Open live WebSocket. Returns { socket, send, close }.
     * @param {string} sessionId
     * @param {{ onMessage?: (msg: object) => void, onError?: (err: Event) => void, WebSocketImpl?: typeof WebSocket }} [opts]
     */
    connectLive(sessionId, opts = {}) {
      const WS = opts.WebSocketImpl ?? globalThis.WebSocket;
      if (!WS) {
        throw new Error("WebSocket not available in this environment");
      }
      const wsUrl = baseUrl.replace(/^http/, "ws") +
        `/v1/sessions/${encodeURIComponent(sessionId)}/live`;
      const socket = new WS(wsUrl);
      socket.addEventListener("message", (ev) => {
        try {
          const msg = JSON.parse(ev.data);
          opts.onMessage?.(msg);
        } catch (e) {
          opts.onError?.(e);
        }
      });
      socket.addEventListener("error", (ev) => opts.onError?.(ev));

      return {
        socket,
        send(msg) {
          socket.send(JSON.stringify(msg));
        },
        tick(dtSecs) {
          this.send({ type: "tick", dtSecs });
        },
        advance(durationSecs) {
          this.send({ type: "advance", durationSecs });
        },
        command(command) {
          this.send({ type: "command", command });
        },
        ping() {
          this.send({ type: "ping" });
        },
        close() {
          socket.close();
        },
      };
    },
  };
}

export {
  presentGrid,
  presentHydro,
  presentSnapshot,
} from "./energySimPresent.js";
