/**
 * Steady power estimate mirroring energy-sim-core (for teaching display).
 */

import type { DerivedGeometry } from "./compileSite";
import type { OperatorInputs, PlantParams } from "./plantParams";

export type EquationStep = {
  /** Symbolic form */
  symbol: string;
  /** Same equation with numbers substituted */
  numeric: string;
};

export type PowerBreakdown = {
  idealElectricalKw: number;
  uncappedElectricalKw: number;
  electricalKw: number;
  hydraulicKw: number;
  flowM3s: number;
  netHeadM: number;
  headLossM: number;
  grossHeadM: number;
  capped: boolean;
  ratedKw: number;
  steps: EquationStep[];
};

function velocityHead(flow: number, diameter: number, g: number): number {
  if (diameter <= 0 || flow <= 0 || g <= 0) return 0;
  const area = Math.PI * (diameter * 0.5) ** 2;
  const v = flow / area;
  return (v * v) / (2 * g);
}

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

  const idealHydW = rho * g * qAvail * Math.max(0, Hgross);
  const idealElectricalKw = (idealHydW / 1000) * eta;

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

  const f1 = (n: number) => n.toFixed(1);
  const f2 = (n: number) => n.toFixed(2);
  const f4 = (n: number) => n.toFixed(4);

  const steps: EquationStep[] = [
    {
      symbol: "H_net = H_gross − H_loss",
      numeric: `H_net = ${f1(Hgross)} − ${f2(loss)} = ${f1(netHead)} m`,
    },
    {
      symbol: "Q = Q_intake × gate × (1 − ½·debris) × (1 − leak)",
      numeric: `Q = ${f4(qAvail)} × ${f2(gate)} × (1 − ${f2(0.5 * clog)}) × (1 − ${f2(leak)}) = ${f4(flow)} m³/s`,
    },
    {
      symbol: "P_h = ρ · g · Q · H_net",
      numeric: `P_h = ${f0(rho)} · ${f2(g)} · ${f4(flow)} · ${f1(netHead)} = ${f2(hydraulicKw)} kW`,
    },
    {
      symbol: "P_e = η_t · η_g · P_h",
      numeric:
        `P_e = ${f2(etaT)} · ${f2(etaG)} · ${f2(hydraulicKw)} = ${f2(uncapped)} kW` +
        (capped ? ` → capped at ${f1(rated)} kW → ${f1(electricalKw)} kW` : ""),
    },
  ];

  return {
    idealElectricalKw,
    uncappedElectricalKw: uncapped,
    electricalKw,
    hydraulicKw,
    flowM3s: flow,
    netHeadM: netHead,
    headLossM: loss,
    grossHeadM: Hgross,
    capped,
    ratedKw: rated,
    steps,
  };
}

function f0(n: number): string {
  return n.toFixed(0);
}
