import type { Snapshot } from "./energySimClient";

export type TrialSample = {
  simTimeS: number;
  electricalPowerKw: number;
  targetElectricalPowerKw: number;
  turbineSpeedRpm: number;
  targetTurbineSpeedRpm: number;
  flowM3s: number;
  netHeadM: number;
};

export type TrialResult = {
  sessionId: string;
  samples: TrialSample[];
  snapshot: Snapshot;
  energyIntervalKwh: number;
  durationSecs: number;
  mode: "interval" | "spinup";
};

export function samplesFromHistory(raw: unknown): TrialSample[] {
  const hist = raw as { samples?: Array<Record<string, unknown>> };
  const list = hist?.samples ?? [];
  return list.map((s) => ({
    simTimeS: num(s.simTimeS),
    electricalPowerKw: num(s.electricalPowerKw),
    targetElectricalPowerKw: num(s.targetElectricalPowerKw),
    turbineSpeedRpm: num(s.turbineSpeedRpm),
    targetTurbineSpeedRpm: num(s.targetTurbineSpeedRpm),
    flowM3s: num(s.flowM3s),
    netHeadM: num(s.netHeadM),
  }));
}

function num(v: unknown): number {
  const n = Number(v);
  return Number.isFinite(n) ? n : 0;
}
