/**
 * Compile clean-slate site geometry + plant params into engine plant JSON.
 *
 * Path length: sum of 3D segment lengths using (s, z) as plan distance and elevation.
 * Gross head: max(0, z_intake - z_turbine).
 * Minor K: base entrance K + bend contributions from turn angle at interior vertices.
 */

import { isSiteComplete, profilePoints, type Site, type SitePoint } from "./site";
import type { HydroPlantJson, PlantParams } from "./plantParams";

/** Reference K for a 90° bend; scales with (θ / 90°)^2. */
export const K_BEND_90 = 0.3;

export type DerivedGeometry = {
  grossHeadM: number;
  lengthM: number;
  baseMinorK: number;
  bendMinorK: number;
  minorLossCoefficient: number;
  bendAnglesDeg: number[];
  segmentLengthsM: number[];
};

export type CompileResult =
  | {
      ok: true;
      plant: HydroPlantJson;
      derived: DerivedGeometry;
      usedOverrides: { head: boolean; length: boolean; minorK: boolean };
    }
  | { ok: false; error: string };

export function segmentLengthM(a: SitePoint, b: SitePoint): number {
  const ds = b.sM - a.sM;
  const dz = b.zM - a.zM;
  return Math.hypot(ds, dz);
}

export function pathLengthM(points: SitePoint[]): number {
  if (points.length < 2) return 0;
  let L = 0;
  for (let i = 1; i < points.length; i++) {
    L += segmentLengthM(points[i - 1]!, points[i]!);
  }
  return L;
}

export function grossHeadM(intake: SitePoint, turbine: SitePoint): number {
  return Math.max(0, intake.zM - turbine.zM);
}

/**
 * Interior turn angle (radians) at vertex b for path a→b→c.
 * 0 = straight, π = full reverse.
 */
export function turnAngleRad(a: SitePoint, b: SitePoint, c: SitePoint): number {
  const v1x = b.sM - a.sM;
  const v1z = b.zM - a.zM;
  const v2x = c.sM - b.sM;
  const v2z = c.zM - b.zM;
  const n1 = Math.hypot(v1x, v1z);
  const n2 = Math.hypot(v2x, v2z);
  if (n1 < 1e-12 || n2 < 1e-12) return 0;
  let cos = (v1x * v2x + v1z * v2z) / (n1 * n2);
  cos = Math.min(1, Math.max(-1, cos));
  return Math.acos(cos);
}

/** Extra minor-loss K for a single turn angle (radians). */
export function bendLossK(turnRad: number): number {
  const halfPi = Math.PI / 2;
  const ratio = turnRad / halfPi;
  return K_BEND_90 * ratio * ratio;
}

export function deriveGeometry(site: Site, baseMinorK: number): DerivedGeometry | null {
  if (!isSiteComplete(site) || !site.intake || !site.turbine) return null;
  const points = profilePoints(site);
  const segmentLengthsM: number[] = [];
  for (let i = 1; i < points.length; i++) {
    segmentLengthsM.push(segmentLengthM(points[i - 1]!, points[i]!));
  }
  const bendAnglesDeg: number[] = [];
  let bendMinorK = 0;
  for (let i = 1; i < points.length - 1; i++) {
    const ang = turnAngleRad(points[i - 1]!, points[i]!, points[i + 1]!);
    bendAnglesDeg.push((ang * 180) / Math.PI);
    bendMinorK += bendLossK(ang);
  }
  return {
    grossHeadM: grossHeadM(site.intake, site.turbine),
    lengthM: pathLengthM(points),
    baseMinorK,
    bendMinorK,
    minorLossCoefficient: baseMinorK + bendMinorK,
    bendAnglesDeg,
    segmentLengthsM,
  };
}

export function compileSite(site: Site, params: PlantParams): CompileResult {
  if (!isSiteComplete(site)) {
    return { ok: false, error: "Place both an intake and a turbine to compile a plant." };
  }
  const derived = deriveGeometry(site, params.penstock.baseMinorLossCoefficient);
  if (!derived) {
    return { ok: false, error: "Could not derive geometry from site." };
  }

  // Geometry is owned by the Layout tab — never honor field overrides here.
  const head = derived.grossHeadM;
  const length = derived.lengthM;
  const minorK = derived.minorLossCoefficient;

  if (!(length > 0)) {
    return { ok: false, error: "Penstock length must be positive (separate intake and turbine)." };
  }
  if (!(params.penstock.diameterM > 0)) {
    return { ok: false, error: "Penstock diameter must be positive." };
  }

  const plant: HydroPlantJson = {
    schemaVersion: 1,
    kind: "hydro-plant",
    id: params.id || "lab-plant",
    label: params.label || undefined,
    stream: {
      availableFlowM3s: params.stream.availableFlowM3s,
    },
    penstock: {
      grossHeadM: head,
      lengthM: length,
      diameterM: params.penstock.diameterM,
      frictionFactor: params.penstock.frictionFactor,
      minorLossCoefficient: minorK,
    },
    turbine: {
      efficiency: params.turbine.efficiency,
      // designFlow only affects teaching speed curve / soft warnings, not a hard power limit.
      designFlowM3s: params.turbine.designFlowM3s,
      // Omit maxSafeFlowM3s so the lab does not hard-cap flow — users explore free configs.
      designSpeedRpm: params.turbine.designSpeedRpm,
      dynamics: { ...params.turbine.dynamics },
    },
    generator: {
      efficiency: params.generator.efficiency,
      // Engine requires a number; use a huge nameplate so electrical output is not clipped.
      ratedPowerKw: 1e9,
    },
    fluid: {
      densityKgM3: params.fluid.densityKgM3,
      gravityMs2: params.fluid.gravityMs2,
    },
  };

  return {
    ok: true,
    plant,
    derived,
    usedOverrides: { head: false, length: false, minorK: false },
  };
}

/**
 * Build a minimal two-point site from plant penstock fields (import helper).
 * Places intake at (0, head) and turbine at (horizontal, 0) where
 * horizontal = sqrt(max(0, L^2 - head^2)).
 */
export function siteFromPlantGeometry(grossHeadM: number, lengthM: number): Site {
  const head = Math.max(0, grossHeadM);
  const L = Math.max(head, lengthM);
  const horiz = Math.sqrt(Math.max(0, L * L - head * head));
  return {
    intake: { sM: 0, zM: head },
    bends: [],
    turbine: { sM: horiz > 1e-9 ? horiz : L, zM: 0 },
  };
}
