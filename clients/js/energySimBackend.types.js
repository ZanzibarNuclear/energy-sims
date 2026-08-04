/**
 * JSDoc typedefs for the EnergySim host adapter (documentation only).
 *
 * @typedef {object} EnergySimBackend
 * @property {'http'|'wasm'} [kind]
 * @property {string} [baseUrl]
 * @property {() => Promise<{ok: boolean, engine?: string}>} [health]
 * @property {(config: string|object) => Promise<{sessionId: string, snapshot: object}>} createSession
 * @property {(checkpoint: string|object) => Promise<{sessionId: string, snapshot: object}>} [createSessionFromCheckpoint]
 * @property {(sessionId: string) => Promise<object>} getSnapshot
 * @property {(sessionId: string) => Promise<object>} start
 * @property {(sessionId: string) => Promise<object>} stop
 * @property {(sessionId: string, body: {durationSecs: number, commands?: object[]}) => Promise<object>} advance
 * @property {(sessionId: string, body: {dtSecs: number}) => Promise<object>} tick
 * @property {(sessionId: string, commands: object[]) => Promise<object>} applyCommands
 * @property {(sessionId: string, id: string, drawing: boolean) => Promise<object>} setLoad
 * @property {(sessionId: string, fields: object) => Promise<object>} setHydroInput
 * @property {(sessionId: string, opts?: {fromSecs?: number, toSecs?: number}) => Promise<{events: object[], samples: object[]}>} history
 * @property {(sessionId: string) => Promise<object>} checkpoint
 * @property {(sessionId: string) => void} [dispose]
 * @property {(sessionId: string, opts?: object) => object} [connectLive]
 */

export {};
