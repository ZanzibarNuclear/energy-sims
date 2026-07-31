/**
 * Teaching defaults and ranges for the simplified Equipment tab.
 *
 * Design choices (user-facing):
 * - Intake flow Q — how much water you divert
 * - Penstock diameter D — main pipe sizing tradeoff (head loss vs cost)
 * - Overall plant efficiency η — η_t × η_g lumped for teaching
 *
 * Consequences of Layout (not Equipment):
 * - Gross head, pipe length, bend minor-loss K
 *
 * Hidden / fixed for now (not design surface):
 * - Friction factor f, base entrance K
 * - Debris, leakage (scenario knobs later)
 * - Gate (operational — stays full open until Run needs it)
 * - Ramps, design speed, nameplate (engine/sim detail)
 */

/** Target mean velocity for a “comfortable” micro-hydro penstock (m/s). */
export const TARGET_VELOCITY_MS = 1.5;

export const FLOW_MIN_M3S = 0.005; // 5 L/s
export const FLOW_MAX_M3S = 0.2; // 200 L/s
export const FLOW_STEP_M3S = 0.005;

export const DIAMETER_MIN_M = 0.08; // 8 cm — below this, default Q eats all head
export const DIAMETER_MAX_M = 0.6; // 60 cm
export const DIAMETER_STEP_M = 0.01;

export const ETA_MIN = 0.4;
export const ETA_MAX = 0.95;
export const ETA_STEP = 0.01;

export const FIXED_FRICTION = 0.02;
export const FIXED_BASE_MINOR_K = 0.5;

/** Diameter that yields target velocity for flow Q: A = Q/v, D = 2√(A/π). */
export function suggestedDiameterM(flowM3s: number, velocityMs = TARGET_VELOCITY_MS): number {
  const q = Math.max(flowM3s, 1e-9);
  const v = Math.max(velocityMs, 0.1);
  const area = q / v;
  const d = 2 * Math.sqrt(area / Math.PI);
  return clamp(d, DIAMETER_MIN_M, DIAMETER_MAX_M);
}

export function meanVelocityMs(flowM3s: number, diameterM: number): number {
  if (diameterM <= 0 || flowM3s <= 0) return 0;
  const area = Math.PI * (diameterM * 0.5) ** 2;
  return flowM3s / area;
}

export function clamp(n: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, n));
}

export function m3sToLs(q: number): number {
  return q * 1000;
}

export function lsToM3s(ls: number): number {
  return ls / 1000;
}
