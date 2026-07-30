/**
 * Clean-slate site geometry for the hydro config lab.
 *
 * - sM = horizontal distance (plan meters along the ground map)
 * - zM = elevation (absolute meters)
 *
 * Pipe length is NOT s; it is the path length along intake → bends → turbine
 * in the s–z plane (straight run ≈ triangle hypotenuse).
 */

export type SitePoint = {
  /** Horizontal distance (m) — plan / map coordinate, not pipe length. */
  sM: number;
  /** Elevation (m), absolute. Display may show z relative to turbine. */
  zM: number;
};

export type SiteElementKind = "intake" | "bend" | "turbine";

/** Selection of a site element (bend uses index into site.bends). */
export type SiteSelection =
  | { kind: "intake" }
  | { kind: "turbine" }
  | { kind: "bend"; index: number }
  | null;

export type Site = {
  intake: SitePoint | null;
  /** Intermediate penstock vertices between intake and turbine (order along the run). */
  bends: SitePoint[];
  turbine: SitePoint | null;
};

export type ToolId = "select" | "intake" | "bend" | "turbine";

export function emptySite(): Site {
  return { intake: null, bends: [], turbine: null };
}

export function cloneSite(site: Site): Site {
  return {
    intake: site.intake ? { ...site.intake } : null,
    bends: site.bends.map((b) => ({ ...b })),
    turbine: site.turbine ? { ...site.turbine } : null,
  };
}

/** Ordered profile points intake → bends → turbine (only present pieces). */
export function profilePoints(site: Site): SitePoint[] {
  const pts: SitePoint[] = [];
  if (site.intake) pts.push(site.intake);
  pts.push(...site.bends);
  if (site.turbine) pts.push(site.turbine);
  return pts;
}

/** Site is ready to compile when intake and turbine are placed. */
export function isSiteComplete(site: Site): boolean {
  return site.intake != null && site.turbine != null;
}

export function siteStatusMessage(site: Site): string {
  if (!site.intake && !site.turbine && site.bends.length === 0) {
    return "Clean slate — place an intake to begin.";
  }
  if (!site.intake) return "Place an intake (upstream diversion / headworks).";
  if (!site.turbine) return "Place a turbine / powerhouse to complete the run.";
  if (site.bends.length === 0) {
    return "Straight penstock ready — optional: add bends, then refine elevations.";
  }
  return `Penstock with ${site.bends.length} bend${site.bends.length === 1 ? "" : "s"} — drag points to refine.`;
}

export function getSelectedPoint(site: Site, sel: SiteSelection): SitePoint | null {
  if (!sel) return null;
  if (sel.kind === "intake") return site.intake;
  if (sel.kind === "turbine") return site.turbine;
  return site.bends[sel.index] ?? null;
}

export function setSelectedPoint(site: Site, sel: SiteSelection, point: SitePoint): Site {
  if (!sel) return site;
  const next = cloneSite(site);
  if (sel.kind === "intake") {
    next.intake = { ...point };
  } else if (sel.kind === "turbine") {
    next.turbine = { ...point };
  } else if (sel.index >= 0 && sel.index < next.bends.length) {
    next.bends[sel.index] = { ...point };
  }
  return next;
}

export function deleteSelection(site: Site, sel: SiteSelection): { site: Site; selection: SiteSelection } {
  if (!sel) return { site, selection: null };
  const next = cloneSite(site);
  if (sel.kind === "intake") {
    next.intake = null;
    return { site: next, selection: null };
  }
  if (sel.kind === "turbine") {
    next.turbine = null;
    return { site: next, selection: null };
  }
  if (sel.index >= 0 && sel.index < next.bends.length) {
    next.bends.splice(sel.index, 1);
  }
  return { site: next, selection: null };
}

/**
 * Insert a bend. Bends stay sorted by horizontal distance s.
 */
export function addBend(site: Site, point: SitePoint): { site: Site; selection: SiteSelection } {
  const next = cloneSite(site);
  next.bends.push({ ...point });
  next.bends.sort((a, b) => a.sM - b.sM);
  const index = next.bends.findIndex((b) => b.sM === point.sM && b.zM === point.zM);
  return {
    site: next,
    selection: { kind: "bend", index: index >= 0 ? index : next.bends.length - 1 },
  };
}

export function placeIntake(site: Site, point: SitePoint): { site: Site; selection: SiteSelection } {
  const next = cloneSite(site);
  next.intake = { ...point };
  return { site: next, selection: { kind: "intake" } };
}

export function placeTurbine(site: Site, point: SitePoint): { site: Site; selection: SiteSelection } {
  const next = cloneSite(site);
  next.turbine = { ...point };
  return { site: next, selection: { kind: "turbine" } };
}

/** Round coords for display / form fields. */
export function roundCoord(n: number, digits = 2): number {
  const f = 10 ** digits;
  return Math.round(n * f) / f;
}

export function selectionLabel(sel: SiteSelection): string {
  if (!sel) return "Nothing selected";
  if (sel.kind === "intake") return "Intake";
  if (sel.kind === "turbine") return "Turbine";
  return `Bend ${sel.index + 1}`;
}
