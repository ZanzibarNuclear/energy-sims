/**
 * Lab document format for save/load (includes site geometry metadata).
 * Export plant JSON uses engine schema only (no lab wrapper).
 */

import { compileSite, siteFromPlantGeometry } from "./compileSite";
import {
  defaultOperator,
  defaultPlantParams,
  type HydroPlantJson,
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
  savedAt: string;
  site: Site;
  params: PlantParams;
  operator: OperatorInputs;
};

export function createLabDocument(
  name: string,
  site: Site,
  params: PlantParams,
  operator: OperatorInputs,
): LabDocument {
  return {
    schemaVersion: LAB_DOC_VERSION,
    kind: LAB_DOC_KIND,
    name: name.trim() || "Untitled site",
    savedAt: new Date().toISOString(),
    site: cloneSite(site),
    params: structuredClone(params),
    operator: structuredClone(operator),
  };
}

export function isLabDocument(value: unknown): value is LabDocument {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  return v.kind === LAB_DOC_KIND && v.schemaVersion === LAB_DOC_VERSION && !!v.site;
}

export function isHydroPlantJson(value: unknown): value is HydroPlantJson {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  return v.kind === "hydro-plant" && typeof v.penstock === "object" && v.penstock != null;
}

/** Parse import: lab document or bare plant JSON. */
export function parseImport(text: string): {
  name: string;
  site: Site;
  params: PlantParams;
  operator: OperatorInputs;
} {
  const data = JSON.parse(text) as unknown;
  if (isLabDocument(data)) {
    return {
      name: data.name,
      site: normalizeSite(data.site),
      params: { ...defaultPlantParams(), ...data.params },
      operator: { ...defaultOperator(), ...data.operator },
    };
  }
  if (isHydroPlantJson(data)) {
    return plantJsonToLabState(data);
  }
  throw new Error('Unrecognized JSON: expected kind "hydro-config-lab" or "hydro-plant".');
}

export function plantJsonToLabState(plant: HydroPlantJson): {
  name: string;
  site: Site;
  params: PlantParams;
  operator: OperatorInputs;
} {
  const params = defaultPlantParams();
  params.id = plant.id || params.id;
  params.label = plant.label || params.label;
  params.stream.availableFlowM3s = plant.stream?.availableFlowM3s ?? params.stream.availableFlowM3s;
  params.penstock.diameterM = plant.penstock.diameterM;
  params.penstock.frictionFactor = plant.penstock.frictionFactor ?? 0.02;
  params.penstock.baseMinorLossCoefficient = plant.penstock.minorLossCoefficient ?? 0.5;
  // Geometry encoded in plant only — reconstruct a simple two-point profile.
  params.penstock.overrideHead = false;
  params.penstock.overrideLength = false;
  params.penstock.overrideMinorLoss = false;
  params.penstock.overrideGrossHeadM = plant.penstock.grossHeadM;
  params.penstock.overrideLengthM = plant.penstock.lengthM;
  params.penstock.overrideMinorLossCoefficient = plant.penstock.minorLossCoefficient ?? 0.5;

  if (plant.turbine) {
    params.turbine.efficiency = plant.turbine.efficiency;
    params.turbine.designFlowM3s = plant.turbine.designFlowM3s;
    params.turbine.maxSafeFlowM3s =
      plant.turbine.maxSafeFlowM3s ?? params.turbine.maxSafeFlowM3s;
    params.turbine.designSpeedRpm =
      plant.turbine.designSpeedRpm ?? params.turbine.designSpeedRpm;
    if (plant.turbine.dynamics) {
      params.turbine.dynamics = { ...params.turbine.dynamics, ...plant.turbine.dynamics };
    }
  }
  if (plant.generator) {
    params.generator.efficiency = plant.generator.efficiency;
    params.generator.ratedPowerKw = plant.generator.ratedPowerKw;
  }
  if (plant.fluid) {
    params.fluid.densityKgM3 = plant.fluid.densityKgM3 ?? params.fluid.densityKgM3;
    params.fluid.gravityMs2 = plant.fluid.gravityMs2 ?? params.fluid.gravityMs2;
  }

  const site = siteFromPlantGeometry(plant.penstock.grossHeadM, plant.penstock.lengthM);
  return {
    name: plant.label || plant.id || "Imported plant",
    site,
    params,
    operator: defaultOperator(),
  };
}

export function exportPlantJson(
  site: Site,
  params: PlantParams,
): { ok: true; plant: HydroPlantJson; text: string } | { ok: false; error: string } {
  const r = compileSite(site, params);
  if (!r.ok) return r;
  return {
    ok: true,
    plant: r.plant,
    text: JSON.stringify(r.plant, null, 2) + "\n",
  };
}

export function downloadText(filename: string, text: string, mime = "application/json") {
  const blob = new Blob([text], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export function emptyLabState(): {
  name: string;
  site: Site;
  params: PlantParams;
  operator: OperatorInputs;
} {
  return {
    name: "Untitled site",
    site: defaultSite(),
    params: defaultPlantParams(),
    operator: defaultOperator(),
  };
}
