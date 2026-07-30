/** Stable world view for the site canvas (meters). */

export type WorldBounds = {
  sMin: number;
  sMax: number;
  zMin: number;
  zMax: number;
};

/** Default teaching view: ~250 m run, ~120 m elevation. */
export const DEFAULT_VIEW: WorldBounds = {
  sMin: -10,
  sMax: 260,
  zMin: -5,
  zMax: 130,
};

export function viewWidth(b: WorldBounds): number {
  return b.sMax - b.sMin;
}

export function viewHeight(b: WorldBounds): number {
  return b.zMax - b.zMin;
}

/** SVG viewBox string with elevation up (flip z). */
export function toSvgViewBox(b: WorldBounds): string {
  return `${b.sMin} ${-b.zMax} ${viewWidth(b)} ${viewHeight(b)}`;
}

/** Nice tick step for a range (meters). */
export function tickStep(range: number): number {
  if (range <= 40) return 5;
  if (range <= 100) return 10;
  if (range <= 250) return 25;
  if (range <= 500) return 50;
  return 100;
}

export function ticks(min: number, max: number, step: number): number[] {
  const start = Math.ceil(min / step) * step;
  const out: number[] = [];
  for (let v = start; v <= max + 1e-9; v += step) {
    out.push(Math.round(v * 1000) / 1000);
  }
  return out;
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

/** Marker radius in world meters so symbols stay readable. */
export function markerRadiusM(b: WorldBounds): number {
  return Math.max(2.5, Math.min(viewWidth(b), viewHeight(b)) * 0.012);
}
