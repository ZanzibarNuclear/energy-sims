/**
 * Steady power estimate mirroring energy-sim-core (for teaching display).
 * Lab path: no max-safe flow clip and no generator nameplate clip.
 */

import type { DerivedGeometry } from "./compileSite";
import type { OperatorInputs, PlantParams } from "./plantParams";

export type EquationStep = {
  id: string;
  factorsSymbol: string[];
  factorsNumeric: string[];
  result: string;
  lhs: string;
};

export type LossBreakdown = {
  frictionM: number;
  /** Entrance / base K contribution */
  entranceM: number;
  /** Bend K contribution (from Layout turn angles) */
  bendM: number;
  /** entrance + bend (total minor) */
  minorM: number;
  debrisM: number;
  totalM: number;
  velocityMs: number;
  velocityHeadM: number;
  baseMinorK: number;
  bendMinorK: number;
};

export type PowerBreakdown = {
  idealElectricalKw: number;
  withLossesKw: number;
  flowM3s: number;
  grossHeadM: number;
  netHeadM: number;
  pipeLengthM: number;
  losses: LossBreakdown;
  /** True when friction/minor losses consume all gross head. */
  headStarved: boolean;
  steps: EquationStep[];
};

function areaM2(diameterM: number): number {
  return Math.PI * (diameterM * 0.5) ** 2;
}

function lossBreakdown(
  flow: number,
  lengthM: number,
  diameterM: number,
  frictionFactor: number,
  baseMinorK: number,
  bendMinorK: number,
  debrisClog: number,
  g: number,
): LossBreakdown {
  const clog = Math.min(1, Math.max(0, debrisClog));
  if (diameterM <= 0 || flow <= 0 || g <= 0) {
    return {
      frictionM: 0,
      entranceM: 0,
      bendM: 0,
      minorM: 0,
      debrisM: 0,
      totalM: 0,
      velocityMs: 0,
      velocityHeadM: 0,
      baseMinorK,
      bendMinorK,
    };
  }
  const a = areaM2(diameterM);
  const v = flow / a;
  const vh = (v * v) / (2 * g);
  const frictionM = frictionFactor * (lengthM / diameterM) * vh;
  const entranceM = baseMinorK * vh;
  const bendM = bendMinorK * vh;
  const minorM = entranceM + bendM;
  const debrisM = 10 * clog * vh;
  return {
    frictionM,
    entranceM,
    bendM,
    minorM,
    debrisM,
    totalM: frictionM + minorM + debrisM,
    velocityMs: v,
    velocityHeadM: vh,
    baseMinorK,
    bendMinorK,
  };
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
  const baseK = derived.baseMinorK;
  const bendK = derived.bendMinorK;
  const qAvail = params.stream.availableFlowM3s;
  const gate = Math.min(1, Math.max(0, operator.gateOpening));
  const leak = Math.min(1, Math.max(0, operator.leakageFraction));
  const clog = Math.min(1, Math.max(0, operator.debrisClogFraction));

  const idealHydW = rho * g * qAvail * Math.max(0, Hgross);
  const idealElectricalKw = (idealHydW / 1000) * eta;

  const flow = qAvail * gate * (1 - 0.5 * clog) * (1 - leak);
  const losses = lossBreakdown(flow, L, D, f, baseK, bendK, clog, g);
  const netHead = Math.max(0, Hgross - losses.totalM);
  const headStarved = losses.totalM > Hgross && flow > 0 && Hgross >= 0;
  const hydraulicKw = (rho * g * flow * netHead) / 1000;
  const withLossesKw = hydraulicKw * eta;

  const f1 = (n: number) => n.toFixed(1);
  const f2 = (n: number) => n.toFixed(2);
  const f4 = (n: number) => n.toFixed(4);

  // Order: Q first (sets velocity), then H_net (loss depends on Q), then power.
  const steps: EquationStep[] = [
    {
      id: "Q",
      lhs: "Q",
      factorsSymbol: ["Qintake", "gate", "(1−½·debris)", "(1−leak)"],
      factorsNumeric: [f4(qAvail), f2(gate), f2(1 - 0.5 * clog), f2(1 - leak)],
      result: `${f4(flow)} m³/s`,
    },
    {
      id: "Hnet",
      lhs: "Hnet",
      factorsSymbol: ["Hgross", "−", "Hloss"],
      factorsNumeric: [f1(Hgross), "−", f2(losses.totalM)],
      result: `${f1(netHead)} m`,
    },
    {
      id: "Ph",
      lhs: "Ph",
      factorsSymbol: ["ρ", "g", "Q", "Hnet"],
      factorsNumeric: [String(Math.round(rho)), f2(g), f4(flow), f1(netHead)],
      result: `${f1(hydraulicKw)} kW`,
    },
    {
      id: "Pe",
      lhs: "Pe",
      factorsSymbol: ["ηt", "ηg", "Ph"],
      factorsNumeric: [f2(etaT), f2(etaG), f1(hydraulicKw)],
      result: `${f1(withLossesKw)} kW`,
    },
  ];

  return {
    idealElectricalKw,
    withLossesKw,
    flowM3s: flow,
    grossHeadM: Hgross,
    netHeadM: netHead,
    pipeLengthM: L,
    losses,
    headStarved,
    steps,
  };
}
