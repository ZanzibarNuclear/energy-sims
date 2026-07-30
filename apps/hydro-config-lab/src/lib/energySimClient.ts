/**
 * Typed client for energy-sim-server (REST + optional WebSocket).
 * Port of clients/js/energySimClient.js for the hydro config lab.
 */

export type EnergySimClientOptions = {
  baseUrl?: string;
  fetchImpl?: typeof fetch;
};

export type Snapshot = {
  simTimeS?: number;
  phase?: string;
  plantId?: string;
  electricalPowerKw?: number;
  targetElectricalPowerKw?: number;
  turbineSpeedRpm?: number;
  availableGenerationKw?: number;
  totalLoadKw?: number;
  marginKw?: number;
  busEnergized?: boolean;
  gridStatus?: string;
  energyGeneratedKwh?: number;
  warnings?: string[];
  [key: string]: unknown;
};

export type HealthResponse = {
  ok?: boolean;
  engine?: string;
  [key: string]: unknown;
};

export type CreateSessionResponse = {
  sessionId: string;
  snapshot?: Snapshot;
};

export type PresentSnapshot = {
  simTimeS?: number;
  electricalPowerKw?: number;
  targetElectricalPowerKw?: number;
  turbineSpeedRpm?: number;
  availableGenerationKw?: number;
  totalLoadKw?: number;
  marginKw?: number;
  busEnergized?: boolean;
  gridStatus?: string;
  brownout: boolean;
  lightLevel: number;
  energyGeneratedKwh?: number;
  warnings: string[];
};

export class EnergySimHttpError extends Error {
  status: number;
  body: unknown;

  constructor(message: string, status: number, body: unknown) {
    super(message);
    this.name = "EnergySimHttpError";
    this.status = status;
    this.body = body;
  }
}

export function createEnergySimClient(options: EnergySimClientOptions = {}) {
  const baseUrl = (options.baseUrl ?? "http://127.0.0.1:8787").replace(/\/$/, "");
  const fetchImpl = options.fetchImpl ?? globalThis.fetch;
  if (typeof fetchImpl !== "function") {
    throw new Error("energySimClient requires fetch (browser or Node 18+)");
  }

  async function request(path: string, init: RequestInit = {}): Promise<unknown> {
    const res = await fetchImpl(`${baseUrl}${path}`, {
      ...init,
      headers: {
        Accept: "application/json",
        ...(init.body ? { "Content-Type": "application/json" } : {}),
        ...(init.headers ?? {}),
      },
    });
    const text = await res.text();
    let body: unknown = null;
    if (text) {
      try {
        body = JSON.parse(text);
      } catch {
        body = text;
      }
    }
    if (!res.ok) {
      const msg =
        (body && typeof body === "object" && "error" in body
          ? String((body as { error: unknown }).error)
          : null) ??
        res.statusText ??
        `HTTP ${res.status}`;
      throw new EnergySimHttpError(msg, res.status, body);
    }
    return body;
  }

  return {
    baseUrl,

    health(): Promise<HealthResponse> {
      return request("/health") as Promise<HealthResponse>;
    },

    createSession(config: string | object): Promise<CreateSessionResponse> {
      const body = typeof config === "string" ? config : JSON.stringify(config);
      return request("/v1/sessions", { method: "POST", body }) as Promise<CreateSessionResponse>;
    },

    getSnapshot(sessionId: string): Promise<Snapshot> {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}`) as Promise<Snapshot>;
    },

    start(sessionId: string): Promise<Snapshot> {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}/start`, {
        method: "POST",
      }) as Promise<Snapshot>;
    },

    stop(sessionId: string): Promise<Snapshot> {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}/stop`, {
        method: "POST",
      }) as Promise<Snapshot>;
    },

    advance(
      sessionId: string,
      body: { durationSecs: number; commands?: object[] },
    ): Promise<unknown> {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}/advance`, {
        method: "POST",
        body: JSON.stringify(body),
      });
    },

    tick(sessionId: string, body: { dtSecs: number }): Promise<Snapshot> {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}/tick`, {
        method: "POST",
        body: JSON.stringify(body),
      }) as Promise<Snapshot>;
    },

    applyCommands(sessionId: string, commands: object[]): Promise<Snapshot> {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}/commands`, {
        method: "POST",
        body: JSON.stringify({ commands }),
      }) as Promise<Snapshot>;
    },

    setLoad(sessionId: string, id: string, drawing: boolean): Promise<Snapshot> {
      return this.applyCommands(sessionId, [
        { type: "set_load", id, drawing: Boolean(drawing) },
      ]);
    },

    setHydroInput(sessionId: string, fields: Record<string, unknown>): Promise<Snapshot> {
      return this.applyCommands(sessionId, [{ type: "set_hydro_input", ...fields }]);
    },

    history(
      sessionId: string,
      range: { fromSecs?: number; toSecs?: number } = {},
    ): Promise<unknown> {
      const q = new URLSearchParams();
      if (range.fromSecs != null) q.set("fromSecs", String(range.fromSecs));
      if (range.toSecs != null) q.set("toSecs", String(range.toSecs));
      const qs = q.toString();
      return request(
        `/v1/sessions/${encodeURIComponent(sessionId)}/history${qs ? `?${qs}` : ""}`,
      );
    },

    checkpoint(sessionId: string): Promise<unknown> {
      return request(`/v1/sessions/${encodeURIComponent(sessionId)}/checkpoint`, {
        method: "POST",
      });
    },

    connectLive(
      sessionId: string,
      opts: {
        onMessage?: (msg: object) => void;
        onError?: (err: Event | Error) => void;
        WebSocketImpl?: typeof WebSocket;
      } = {},
    ) {
      const WS = opts.WebSocketImpl ?? globalThis.WebSocket;
      if (!WS) {
        throw new Error("WebSocket not available in this environment");
      }
      const wsUrl =
        baseUrl.replace(/^http/, "ws") +
        `/v1/sessions/${encodeURIComponent(sessionId)}/live`;
      const socket = new WS(wsUrl);
      socket.addEventListener("message", (ev) => {
        try {
          const msg = JSON.parse(String((ev as MessageEvent).data)) as object;
          opts.onMessage?.(msg);
        } catch (e) {
          opts.onError?.(e instanceof Error ? e : new Error(String(e)));
        }
      });
      socket.addEventListener("error", (ev) => opts.onError?.(ev));

      return {
        socket,
        send(msg: object) {
          socket.send(JSON.stringify(msg));
        },
        tick(dtSecs: number) {
          this.send({ type: "tick", dtSecs });
        },
        advance(durationSecs: number) {
          this.send({ type: "advance", durationSecs });
        },
        command(command: object) {
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

export type EnergySimClient = ReturnType<typeof createEnergySimClient>;

/** Map a Snapshot into control-room / lab friendly fields. */
export function presentSnapshot(snapshot: Snapshot | null | undefined): PresentSnapshot | null {
  if (!snapshot) return null;
  const brownout =
    snapshot.gridStatus === "brownout" || snapshot.gridStatus === "shortage";
  return {
    simTimeS: snapshot.simTimeS,
    electricalPowerKw: snapshot.electricalPowerKw,
    targetElectricalPowerKw: snapshot.targetElectricalPowerKw,
    turbineSpeedRpm: snapshot.turbineSpeedRpm,
    availableGenerationKw: snapshot.availableGenerationKw,
    totalLoadKw: snapshot.totalLoadKw,
    marginKw: snapshot.marginKw,
    busEnergized: snapshot.busEnergized,
    gridStatus: snapshot.gridStatus,
    brownout,
    lightLevel: brownout ? 0.4 : snapshot.busEnergized ? 1.0 : 0.0,
    energyGeneratedKwh: snapshot.energyGeneratedKwh,
    warnings: snapshot.warnings ?? [],
  };
}

/** Default engine URL for local lab work. */
export function defaultEngineUrl(): string {
  const fromEnv = import.meta.env.VITE_ENERGY_SIM_URL as string | undefined;
  return (fromEnv && fromEnv.trim()) || "http://127.0.0.1:8787";
}
