/**
 * Site canvas view helpers.
 *
 * Coordinate meaning (stored on the site model):
 * - sM = horizontal distance (plan / map meters), not pipe length
 * - zM = elevation (absolute meters)
 *
 * Pipe length L is the path length along the penstock (sum of segment
 * lengths in the s–z plane) — the triangle hypotenuse for a straight run.
 *
 * Display: elevation is shown relative to the turbine when one exists
 * (turbine at elev 0). Horizontal s is shown as stored.
 */

export type WorldBounds = {
  sMin: number;
  sMax: number;
  zMin: number;
  zMax: number;
};

/** Square grid step (meters) — same for horizontal and elevation. */
export const GRID_STEP_M = 10;

/**
 * Default view in *display* coordinates (m):
 * horizontal distance × elevation (above turbine when placed).
 */
export const DEFAULT_VIEW: WorldBounds = {
  sMin: -20,
  sMax: 250,
  zMin: -20,
  zMax: 120,
};

export function viewWidth(b: WorldBounds): number {
  return b.sMax - b.sMin;
}

export function viewHeight(b: WorldBounds): number {
  return b.zMax - b.zMin;
}

/** SVG viewBox string with elevation up (flip z). Bounds are display coords. */
export function toSvgViewBox(b: WorldBounds): string {
  return `${b.sMin} ${-b.zMax} ${viewWidth(b)} ${viewHeight(b)}`;
}

export function ticks(min: number, max: number, step: number): number[] {
  const start = Math.ceil(min / step) * step;
  const out: number[] = [];
  for (let v = start; v <= max + 1e-9; v += step) {
    out.push(Math.round(v * 1000) / 1000);
  }
  return out;
}

export function snapToGrid(value: number, step: number = GRID_STEP_M): number {
  return Math.round(value / step) * step;
}

export function snapPoint(
  sM: number,
  zM: number,
  step: number = GRID_STEP_M,
): { sM: number; zM: number } {
  return { sM: snapToGrid(sM, step), zM: snapToGrid(zM, step) };
}

export function clampPoint(
  sM: number,
  zM: number,
  b: WorldBounds,
): { sM: number; zM: number } {
  return {
    sM: Math.min(b.sMax, Math.max(b.sMin, sM)),
    zM: Math.min(b.zMax, Math.max(b.zMin, zM)),
  };
}

/** Marker radius in display meters so symbols stay readable. */
export function markerRadiusM(b: WorldBounds): number {
  return Math.max(2.5, Math.min(viewWidth(b), viewHeight(b)) * 0.012);
}

/**
 * Elevation origin for display: turbine elevation when present, else 0.
 * Head H = intake.z - turbine.z appears as intake's display elevation.
 */
export function elevationOriginZ(turbineZM: number | null | undefined): number {
  return turbineZM ?? 0;
}

export function worldToDisplay(
  sM: number,
  zM: number,
  elevOriginZ: number,
): { s: number; z: number } {
  return { s: sM, z: zM - elevOriginZ };
}

export function displayToWorld(
  s: number,
  z: number,
  elevOriginZ: number,
): { sM: number; zM: number } {
  return { sM: s, zM: z + elevOriginZ };
}
