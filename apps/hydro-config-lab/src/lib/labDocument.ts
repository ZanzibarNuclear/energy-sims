/**
 * Single lab configuration JSON: site layout + equipment + operator.
 * Save downloads this file; Open loads it back.
 */

import {
  defaultOperator,
  defaultPlantParams,
  type OperatorInputs,
  type PlantParams,
} from "./plantParams";
import { cloneSite, defaultSite, normalizeSite, type Site } from "./site";

export const LAB_DOC_KIND = "hydro-config-lab";
export const LAB_DOC_VERSION = 1;

export type LabDocument = {
  schemaVersion: number;
  kind: typeof LAB_DOC_KIND;
  name: string;
  /** ISO timestamp of last Save (file artifact). */
  savedAt: string;
  site: Site;
  params: PlantParams;
  operator: OperatorInputs;
};

export type LabState = {
  name: string;
  site: Site;
  params: PlantParams;
  operator: OperatorInputs;
};

export function createLabDocument(
  name: string,
  site: Site,
  params: PlantParams,
  operator: OperatorInputs,
  savedAt: string = new Date().toISOString(),
): LabDocument {
  return {
    schemaVersion: LAB_DOC_VERSION,
    kind: LAB_DOC_KIND,
    name: name.trim() || "Untitled site",
    savedAt,
    site: cloneSite(site),
    params: structuredClone(params),
    operator: structuredClone(operator),
  };
}

export function isLabDocument(value: unknown): value is LabDocument {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  return v.kind === LAB_DOC_KIND && typeof v.site === "object" && v.site != null;
}

/** Parse a saved configuration file (full lab JSON only). */
export function parseLabFile(text: string): LabState {
  const data = JSON.parse(text) as unknown;
  if (!isLabDocument(data)) {
    throw new Error(
      'Not a hydro config lab file. Expected JSON with kind "hydro-config-lab".',
    );
  }
  return {
    name: data.name || "Untitled site",
    site: normalizeSite(data.site),
    params: { ...defaultPlantParams(), ...data.params },
    operator: { ...defaultOperator(), ...data.operator },
  };
}

/** Filename: site-name-YYYYMMDD-HHMMSS.json */
export function labFilename(name: string, savedAt: string = new Date().toISOString()): string {
  const slug =
    (name.trim() || "untitled-site")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "") || "untitled-site";
  const d = new Date(savedAt);
  const pad = (n: number) => String(n).padStart(2, "0");
  const stamp = Number.isFinite(d.getTime())
    ? `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}-${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}`
    : "saved";
  return `${slug}-${stamp}.json`;
}

export function downloadLabFile(doc: LabDocument): string {
  const filename = labFilename(doc.name, doc.savedAt);
  const text = JSON.stringify(doc, null, 2) + "\n";
  const blob = new Blob([text], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
  return filename;
}

export function emptyLabState(): LabState {
  return {
    name: "Untitled site",
    site: defaultSite(),
    params: defaultPlantParams(),
    operator: defaultOperator(),
  };
}
