/**
 * Steady power estimate mirroring energy-sim-core (for teaching display).
 * Layout supplies H_gross and L; losses follow the engine catalog.
 */

import type { DerivedGeometry } from "./compileSite";
import type { OperatorInputs, PlantParams } from "./plantParams";

export type PowerBreakdown = {
  /** Ideal (no losses): η ρ g Q_intake H_gross */
  idealElectricalKw: number;
  /** With losses & operator, before nameplate cap */
  uncappedElectricalKw: number;
  /** After ratedPowerKw cap */
  electricalKw: number;
  hydraulicKw: number;
  flowM3s: number;
  qAvailable: number;
  grossHeadM: number;
  netHeadM: number;
  headLossM: number;
  etaTurbine: number;
  etaGenerator: number;
  eta: number;
  rho: number;
  g: number;
  ratedKw: number;
  capped: boolean;
  /** Human-readable substitution for the main equation */
  lines: string[];
};

function velocityHead(flow: number, diameter: number, g: number): number {
  if (diameter <= 0 || flow <= 0 || g <= 0) return 0;
  const area = Math.PI * (diameter * 0.5) ** 2;
  const v = flow / area;
  return (v * v) / (2 * g);
}

/** Head losses matching core catalog (friction + minor + debris). */
function headLossM(
  flow: number,
  lengthM: number,
  diameterM: number,
  frictionFactor: number,
  minorK: number,
  debrisClog: number,
  g: number,
): number {
  const clog = Math.min(1, Math.max(0, debrisClog));
  const vh = velocityHead(flow, diameterM, g);
  const friction = diameterM > 0 ? frictionFactor * (lengthM / diameterM) * vh : 0;
  const minor = minorK * vh;
  const debris = 10 * clog * vh;
  return friction + minor + debris;
}

export function computePowerBreakdown(
  params: PlantParams,
  operator: OperatorInputs,
  derived: DerivedGeometry | null,
): PowerBreakdown | null {
  if (!derived || !(derived.lengthM > 0)) return null;

  const rho = params.fluid.densityKgM3;
  const g = params.fluid.gravityMs2;
  const etaT = params.turbine.efficiency;
  const etaG = params.generator.efficiency;
  const eta = etaT * etaG;
  const Hgross = derived.grossHeadM;
  const L = derived.lengthM;
  const D = params.penstock.diameterM;
  const f = params.penstock.frictionFactor;
  const K = derived.minorLossCoefficient;
  const qAvail = params.stream.availableFlowM3s;
  const gate = Math.min(1, Math.max(0, operator.gateOpening));
  const leak = Math.min(1, Math.max(0, operator.leakageFraction));
  const clog = Math.min(1, Math.max(0, operator.debrisClogFraction));

  // Ideal max: full intake flow, full gross head, no losses (teaching ceiling).
  const idealHydW = rho * g * qAvail * Math.max(0, Hgross);
  const idealElectricalKw = (idealHydW / 1000) * eta;

  // Operating point (engine-like steady state).
  let flow = qAvail * gate * (1 - 0.5 * clog) * (1 - leak);
  if (params.turbine.maxSafeFlowM3s != null && flow > params.turbine.maxSafeFlowM3s) {
    flow = params.turbine.maxSafeFlowM3s;
  }
  const loss = headLossM(flow, L, D, f, K, clog, g);
  const netHead = Math.max(0, Hgross - loss);
  const hydraulicKw = (rho * g * flow * netHead) / 1000;
  const uncapped = hydraulicKw * eta;
  const rated = params.generator.ratedPowerKw;
  const capped = uncapped > rated;
  const electricalKw = capped ? rated : uncapped;

  const f3 = (n: number) => n.toFixed(3);
  const f2 = (n: number) => n.toFixed(2);
  const f4 = (n: number) => n.toFixed(4);

  const lines = [
    `P_hyd = ρ g Q H_net = ${f2(rho)} × ${f3(g)} × ${f4(flow)} × ${f2(netHead)} → ${f3(hydraulicKw)} kW`,
    `P_e = η_t η_g P_hyd = ${f2(etaT)} × ${f2(etaG)} × ${f3(hydraulicKw)} → ${f3(uncapped)} kW` +
      (capped ? ` (capped at rated ${f2(rated)} kW → ${f3(electricalKw)} kW)` : ""),
    `H_net = H_gross − H_loss = ${f2(Hgross)} − ${f3(loss)} = ${f2(netHead)} m`,
    `Q = Q_intake × gate × (1 − ½·debris) × (1 − leak) = ${f4(qAvail)} × ${f2(gate)} × … → ${f4(flow)} m³/s`,
  ];

  return {
    idealElectricalKw,
    uncappedElectricalKw: uncapped,
    electricalKw,
    hydraulicKw,
    flowM3s: flow,
    qAvailable: qAvail,
    grossHeadM: Hgross,
    netHeadM: netHead,
    headLossM: loss,
    etaTurbine: etaT,
    etaGenerator: etaG,
    eta,
    rho,
    g,
    ratedKw: rated,
    capped,
    lines,
  };
}
