/**
 * Steady power estimate mirroring energy-sim-core (for teaching display).
 */

import type { DerivedGeometry } from "./compileSite";
import type { OperatorInputs, PlantParams } from "./plantParams";

export type EquationStep = {
  /** Which quantity this step solves for (for a11y). */
  id: string;
  /** Product chain of symbols, e.g. ["ηt", "ηg", "ρ", "g", "Q", "Hnet"] */
  factorsSymbol: string[];
  /** Same chain with numeric values as strings */
  factorsNumeric: string[];
  /** Result value + unit */
  result: string;
  /** Left-hand side label HTML-ish plain text for display */
  lhs: string;
};

export type PowerBreakdown = {
  idealElectricalKw: number;
  uncappedElectricalKw: number;
  electricalKw: number;
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
      id: "Hnet",
      lhs: "Hnet",
      factorsSymbol: ["Hgross", "−", "Hloss"],
      factorsNumeric: [f1(Hgross), "−", f2(loss)],
      result: `${f1(netHead)} m`,
    },
    {
      id: "Q",
      lhs: "Q",
      factorsSymbol: ["Qintake", "gate", "(1−½·debris)", "(1−leak)"],
      factorsNumeric: [f4(qAvail), f2(gate), f2(1 - 0.5 * clog), f2(1 - leak)],
      result: `${f4(flow)} m³/s`,
    },
    {
      id: "Ph",
      lhs: "Ph",
      factorsSymbol: ["ρ", "g", "Q", "Hnet"],
      // ρ g Q H → watts; show kW result (values use SI: ρ,g,Q,H → W, /1000 = kW)
      factorsNumeric: [f0(rho), f2(g), f4(flow), f1(netHead)],
      result: `${f1(hydraulicKw)} kW`,
    },
    {
      id: "Pe",
      lhs: "Pe",
      factorsSymbol: ["ηt", "ηg", "Ph"],
      factorsNumeric: [f2(etaT), f2(etaG), f1(hydraulicKw)],
      result: capped
        ? `${f1(uncapped)} kW → ${f1(electricalKw)} kW (rated)`
        : `${f1(electricalKw)} kW`,
    },
  ];

  return {
    idealElectricalKw,
    uncappedElectricalKw: uncapped,
    electricalKw,
    steps,
  };
}

function f0(n: number): string {
  return n.toFixed(0);
}
