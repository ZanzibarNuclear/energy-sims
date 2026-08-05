/**
 * Map energy-sim Snapshot → control-room / holo presentation strips.
 * Pure helpers; no network or WASM.
 */

/**
 * Hydro sensors terminal strip.
 * @param {object} snapshot
 */
export function presentHydro(snapshot) {
  if (!snapshot) return null;
  return {
    simTimeS: snapshot.simTimeS,
    phase: snapshot.phase,
    plantId: snapshot.plantId,
    flowM3s: snapshot.flowM3s,
    grossHeadM: snapshot.grossHeadM,
    netHeadM: snapshot.netHeadM,
    headLossM: snapshot.headLossM,
    hydraulicPowerKw: snapshot.hydraulicPowerKw,
    electricalPowerKw: snapshot.electricalPowerKw,
    targetElectricalPowerKw: snapshot.targetElectricalPowerKw,
    turbineSpeedRpm: snapshot.turbineSpeedRpm,
    targetTurbineSpeedRpm: snapshot.targetTurbineSpeedRpm,
    energyGeneratedKwh: snapshot.energyGeneratedKwh,
    warnings: snapshot.warnings ?? [],
  };
}

/**
 * Station grid terminal strip (aggregates + load table).
 * @param {object} snapshot
 */
export function presentGrid(snapshot) {
  if (!snapshot) return null;
  const status = snapshot.gridStatus ?? "ok";
  const brownout = status === "brownout" || status === "shortage";
  const loads = (snapshot.loads ?? []).map((row) => ({
    id: row.id,
    label: row.label ?? row.id,
    ratingW: row.ratingW,
    priority: row.priority,
    drawing: Boolean(row.drawing),
  }));
  return {
    simTimeS: snapshot.simTimeS,
    availableGenerationKw: snapshot.availableGenerationKw,
    totalLoadKw: snapshot.totalLoadKw,
    marginKw: snapshot.marginKw,
    busEnergized: Boolean(snapshot.busEnergized),
    gridStatus: status,
    brownout,
    /** Host hint: dim lights when brownout/shortage. */
    lightLevel: brownout ? 0.4 : snapshot.busEnergized ? 1.0 : 0.0,
    loads,
    drawingLoadIds: loads.filter((l) => l.drawing).map((l) => l.id),
  };
}

/**
 * Combined presentation for hosts that want one object.
 * @param {object} snapshot
 */
export function presentSnapshot(snapshot) {
  if (!snapshot) return null;
  const hydro = presentHydro(snapshot);
  const grid = presentGrid(snapshot);
  return {
    ...hydro,
    ...grid,
    hydro,
    grid,
    // Back-compat flat fields (used by earlier clients)
    electricalPowerKw: hydro.electricalPowerKw,
    targetElectricalPowerKw: hydro.targetElectricalPowerKw,
    turbineSpeedRpm: hydro.turbineSpeedRpm,
    availableGenerationKw: grid.availableGenerationKw,
    totalLoadKw: grid.totalLoadKw,
    marginKw: grid.marginKw,
    busEnergized: grid.busEnergized,
    gridStatus: grid.gridStatus,
    brownout: grid.brownout,
    lightLevel: grid.lightLevel,
    energyGeneratedKwh: hydro.energyGeneratedKwh,
    warnings: hydro.warnings,
    loads: grid.loads,
  };
}
