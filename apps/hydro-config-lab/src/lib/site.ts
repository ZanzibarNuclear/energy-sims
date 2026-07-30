/**
 * Clean-slate site geometry for the hydro config lab.
 *
 * - sM = horizontal distance (plan meters along the ground map)
 * - zM = elevation (absolute meters)
 *
 * Pipe length is the path length along intake → bends → turbine in the s–z plane.
 * Default layout: 50 m head, 120 m horizontal run → 130 m straight pipe.
 */

export type SitePoint = {
  /** Horizontal distance (m) — plan / map coordinate, not pipe length. */
  sM: number;
  /** Elevation (m), absolute. Display may show z relative to turbine. */
  zM: number;
};

/** Selection: only bends are selectable for delete; intake/turbine always exist. */
export type SiteSelection = { kind: "bend"; index: number } | null;

export type Site = {
  intake: SitePoint;
  /** Intermediate penstock vertices between intake and turbine (order along the run). */
  bends: SitePoint[];
  turbine: SitePoint;
};

/**
 * Default teaching site:
 * - Intake at s=30 m, z=50 m (50 m above turbine)
 * - Turbine at s=150 m, z=0 m (120 m horizontal from intake)
 * - Straight pipe L = √(120² + 50²) = 130 m
 */
export function defaultSite(): Site {
  return {
    intake: { sM: 30, zM: 50 },
    bends: [],
    turbine: { sM: 150, zM: 0 },
  };
}

/** Coerce partial / legacy docs into a full site. */
export function normalizeSite(raw: {
  intake?: SitePoint | null;
  bends?: SitePoint[];
  turbine?: SitePoint | null;
} | null | undefined): Site {
  const d = defaultSite();
  return {
    intake: raw?.intake ?? d.intake,
    bends: Array.isArray(raw?.bends) ? raw!.bends.map((b) => ({ ...b })) : [],
    turbine: raw?.turbine ?? d.turbine,
  };
}

export function cloneSite(site: Site): Site {
  return {
    intake: { ...site.intake },
    bends: site.bends.map((b) => ({ ...b })),
    turbine: { ...site.turbine },
  };
}

/** Ordered profile points intake → bends → turbine. */
export function profilePoints(site: Site): SitePoint[] {
  return [site.intake, ...site.bends, site.turbine];
}

export function isSiteComplete(site: Site): boolean {
  return site.intake != null && site.turbine != null;
}

export type SegmentInfo = {
  index: number;
  from: SitePoint;
  to: SitePoint;
  lengthM: number;
  /** Horizontal run of this segment (m). */
  runM: number;
  /** Elevation drop (positive downhill toward turbine). */
  dropM: number;
  /** Slope angle in degrees (0 = flat, 90 = vertical). */
  slopeDeg: number;
};

export function segments(site: Site): SegmentInfo[] {
  const pts = profilePoints(site);
  const out: SegmentInfo[] = [];
  for (let i = 1; i < pts.length; i++) {
    const from = pts[i - 1]!;
    const to = pts[i]!;
    const runM = to.sM - from.sM;
    const dropM = from.zM - to.zM;
    const lengthM = Math.hypot(runM, dropM);
    const slopeDeg =
      lengthM < 1e-9 ? 0 : (Math.atan2(Math.abs(dropM), Math.abs(runM)) * 180) / Math.PI;
    out.push({ index: i - 1, from, to, lengthM, runM, dropM, slopeDeg });
  }
  return out;
}

/**
 * Place a new bend at the midpoint of the longest penstock segment
 * (or the single segment if straight). Selects the new bend.
 */
export function addBendOnPenstock(site: Site): { site: Site; selection: SiteSelection } {
  const segs = segments(site);
  if (segs.length === 0) {
    return { site: cloneSite(site), selection: null };
  }
  let best = segs[0]!;
  for (const s of segs) {
    if (s.lengthM > best.lengthM) best = s;
  }
  const mid: SitePoint = {
    sM: (best.from.sM + best.to.sM) / 2,
    zM: (best.from.zM + best.to.zM) / 2,
  };
  const next = cloneSite(site);
  next.bends.push(mid);
  next.bends.sort((a, b) => a.sM - b.sM);
  const index = next.bends.findIndex(
    (b) => Math.abs(b.sM - mid.sM) < 1e-9 && Math.abs(b.zM - mid.zM) < 1e-9,
  );
  return {
    site: next,
    selection: { kind: "bend", index: index >= 0 ? index : next.bends.length - 1 },
  };
}

export function deleteBend(site: Site, index: number): Site {
  if (index < 0 || index >= site.bends.length) return site;
  const next = cloneSite(site);
  next.bends.splice(index, 1);
  return next;
}

export function moveBend(site: Site, index: number, point: SitePoint): Site {
  if (index < 0 || index >= site.bends.length) return site;
  const next = cloneSite(site);
  next.bends[index] = constrainBendPoint(next, index, point);
  next.bends.sort((a, b) => a.sM - b.sM);
  return next;
}

export function moveIntake(site: Site, point: SitePoint): Site {
  const next = cloneSite(site);
  next.intake = constrainIntakePoint(next, point);
  return next;
}

export function moveTurbine(site: Site, point: SitePoint): Site {
  const next = cloneSite(site);
  next.turbine = constrainTurbinePoint(next, point);
  // Keep penstock above the new turbine floor.
  next.intake = {
    ...next.intake,
    zM: Math.max(next.intake.zM, next.turbine.zM),
  };
  next.bends = next.bends.map((b) => ({
    ...b,
    zM: Math.max(b.zM, next.turbine.zM),
  }));
  return enforceMonotonicElevations(next);
}

export function roundCoord(n: number, digits = 2): number {
  const f = 10 ** digits;
  return Math.round(n * f) / f;
}

// ── Penstock gravity-flow rules ──────────────────────────────────────────

/**
 * Real diversion penstocks need a continuous downhill (or flat) run:
 * no high points after the intake (air pockets / reverse slope), and
 * nothing below the turbine elevation.
 */
export type PenstockIssue = {
  code: "uphill" | "below_turbine" | "above_intake";
  message: string;
};

export function validatePenstock(site: Site): PenstockIssue[] {
  const issues: PenstockIssue[] = [];
  const pts = profilePoints(site);
  const zTurbine = site.turbine.zM;
  const zIntake = site.intake.zM;

  for (let i = 0; i < pts.length; i++) {
    const p = pts[i]!;
    if (p.zM < zTurbine - 1e-6) {
      issues.push({
        code: "below_turbine",
        message:
          "A penstock point sits below the turbine. Water would pool there and not reach the machine. Keep every point at or above the turbine floor.",
      });
      break;
    }
  }

  if (pts.some((p, i) => i > 0 && p.zM > zIntake + 1e-6)) {
    issues.push({
      code: "above_intake",
      message:
        "A bend rises above the intake. The free surface / pressure line starts at the intake, so a higher bend would need pumping or create a siphon risk in teaching-scale diversion plants.",
    });
  }

  for (let i = 1; i < pts.length; i++) {
    const prev = pts[i - 1]!;
    const cur = pts[i]!;
    if (cur.zM > prev.zM + 1e-6) {
      issues.push({
        code: "uphill",
        message:
          "The penstock has an uphill stretch. Gravity diversion needs a downhill (or flat) profile along the pipe so the flow does not stall or trap air at a high point.",
      });
      break;
    }
  }

  return issues;
}

/** Clamp a candidate bend so elev is between neighbors and ≥ turbine. */
export function constrainBendPoint(
  site: Site,
  bendIndex: number,
  point: SitePoint,
): SitePoint {
  const pts = profilePoints(site);
  // path index of this bend = 1 + bendIndex
  const pathI = 1 + bendIndex;
  const prev = pts[pathI - 1] ?? site.intake;
  const next = pts[pathI + 1] ?? site.turbine;
  const zHi = Math.min(prev.zM, site.intake.zM);
  const zLo = Math.max(next.zM, site.turbine.zM);
  let zM = Math.min(zHi, Math.max(zLo, point.zM));
  // Keep between neighbors horizontally when possible
  const sLo = Math.min(prev.sM, next.sM);
  const sHi = Math.max(prev.sM, next.sM);
  let sM = point.sM;
  if (sHi - sLo > 1e-6) {
    sM = Math.min(sHi, Math.max(sLo, sM));
  }
  return { sM, zM };
}

export function constrainIntakePoint(site: Site, point: SitePoint): SitePoint {
  // Intake must stay at/above every downstream point and the turbine.
  const minDownstream = Math.max(
    site.turbine.zM,
    ...site.bends.map((b) => b.zM),
  );
  return {
    sM: point.sM,
    zM: Math.max(point.zM, minDownstream),
  };
}

export function constrainTurbinePoint(site: Site, point: SitePoint): SitePoint {
  // Turbine is the low end: not above the nearest upstream point.
  const upstream =
    site.bends.length > 0 ? site.bends[site.bends.length - 1]! : site.intake;
  return {
    sM: point.sM,
    zM: Math.min(point.zM, upstream.zM),
  };
}

/** After bulk edits, walk intake→turbine and push elevations downhill only. */
export function enforceMonotonicElevations(site: Site): Site {
  const next = cloneSite(site);
  const pts = profilePoints(next);
  // Forward: each point ≤ previous
  for (let i = 1; i < pts.length; i++) {
    if (pts[i]!.zM > pts[i - 1]!.zM) {
      pts[i] = { ...pts[i]!, zM: pts[i - 1]!.zM };
    }
  }
  // Backward from turbine floor: each point ≥ turbine
  const zT = pts[pts.length - 1]!.zM;
  for (let i = 0; i < pts.length; i++) {
    if (pts[i]!.zM < zT) {
      pts[i] = { ...pts[i]!, zM: zT };
    }
  }
  next.intake = pts[0]!;
  next.turbine = pts[pts.length - 1]!;
  next.bends = pts.slice(1, -1);
  return next;
}
