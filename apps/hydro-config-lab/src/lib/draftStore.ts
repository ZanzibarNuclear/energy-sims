/**
 * Auto-save the working lab state in the browser so refresh / accidental
 * navigation does not lose work. This is not a user-facing "saved file".
 */

import {
  createLabDocument,
  isLabDocument,
  type LabDocument,
} from "./labDocument";
import type { OperatorInputs, PlantParams } from "./plantParams";
import { normalizeSite, type Site } from "./site";

const DRAFT_KEY = "energy-sims.hydro-config-lab.draft.v1";

export function saveDraft(
  name: string,
  site: Site,
  params: PlantParams,
  operator: OperatorInputs,
): void {
  try {
    const doc = createLabDocument(name, site, params, operator);
    localStorage.setItem(DRAFT_KEY, JSON.stringify(doc));
  } catch {
    // Quota or private mode — ignore
  }
}

export function loadDraft(): LabDocument | null {
  try {
    const raw = localStorage.getItem(DRAFT_KEY);
    if (!raw) return null;
    const data = JSON.parse(raw) as unknown;
    if (!isLabDocument(data)) return null;
    return {
      ...data,
      site: normalizeSite(data.site),
    };
  } catch {
    return null;
  }
}

export function clearDraft(): void {
  try {
    localStorage.removeItem(DRAFT_KEY);
  } catch {
    /* ignore */
  }
}
