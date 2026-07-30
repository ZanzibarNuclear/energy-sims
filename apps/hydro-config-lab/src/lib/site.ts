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
  next.bends[index] = { ...point };
  next.bends.sort((a, b) => a.sM - b.sM);
  return next;
}

export function moveIntake(site: Site, point: SitePoint): Site {
  return { ...cloneSite(site), intake: { ...point } };
}

export function moveTurbine(site: Site, point: SitePoint): Site {
  return { ...cloneSite(site), turbine: { ...point } };
}

export function roundCoord(n: number, digits = 2): number {
  const f = 10 ** digits;
  return Math.round(n * f) / f;
}
