/**
 * Named configurations in the browser (localStorage).
 * Survives restarts in the same browser/profile; not shared across devices.
 */

import {
  createLabDocument,
  isLabDocument,
  type LabDocument,
} from "./labDocument";
import type { OperatorInputs, PlantParams } from "./plantParams";
import { normalizeSite, type Site } from "./site";

const STORAGE_KEY = "energy-sims.hydro-config-lab.configs.v1";

export type ConfigIndexEntry = {
  name: string;
  savedAt: string;
};

type StoreBlob = {
  version: 1;
  /** name → full lab document */
  configs: Record<string, LabDocument>;
};

function readBlob(): StoreBlob {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { version: 1, configs: {} };
    const data = JSON.parse(raw) as StoreBlob;
    if (!data || data.version !== 1 || typeof data.configs !== "object") {
      return { version: 1, configs: {} };
    }
    return data;
  } catch {
    return { version: 1, configs: {} };
  }
}

function writeBlob(blob: StoreBlob) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(blob));
}

export function listConfigs(): ConfigIndexEntry[] {
  return Object.values(readBlob().configs)
    .map((c) => ({ name: c.name, savedAt: c.savedAt }))
    .sort((a, b) => b.savedAt.localeCompare(a.savedAt));
}

export function loadConfig(name: string): LabDocument | null {
  const doc = readBlob().configs[name];
  if (!doc || !isLabDocument(doc)) return null;
  return {
    ...doc,
    site: normalizeSite(doc.site),
  };
}

export function saveConfig(
  name: string,
  site: Site,
  params: PlantParams,
  operator: OperatorInputs,
): LabDocument {
  const key = name.trim() || "Untitled site";
  const doc = createLabDocument(key, site, params, operator);
  const blob = readBlob();
  blob.configs[key] = doc;
  writeBlob(blob);
  return doc;
}

export function deleteConfig(name: string): void {
  const blob = readBlob();
  delete blob.configs[name];
  writeBlob(blob);
}
