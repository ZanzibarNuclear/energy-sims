/**
 * Non-geometry plant parameters and operator inputs for the lab.
 * Geometry (head, length, bend K) comes from the site via compileSite.
 */

export type OperatorInputs = {
  gateOpening: number;
  debrisClogFraction: number;
  leakageFraction: number;
  online: boolean;
};

export type TurbineDynamics = {
  speedRampUpS: number;
  speedRampDownS: number;
  powerRampUpS: number;
  powerRampDownS: number;
};

export type PlantParams = {
  id: string;
  label: string;
  stream: {
    availableFlowM3s: number;
  };
  penstock: {
    diameterM: number;
    frictionFactor: number;
    /** Base minor-loss K (entrance / fittings) before bend contributions. */
    baseMinorLossCoefficient: number;
    /** When true, use overrideMinorLossCoefficient instead of base + bends. */
    overrideMinorLoss: boolean;
    overrideMinorLossCoefficient: number;
    /** When true, ignore derived head and use overrideGrossHeadM. */
    overrideHead: boolean;
    overrideGrossHeadM: number;
    /** When true, ignore derived length and use overrideLengthM. */
    overrideLength: boolean;
    overrideLengthM: number;
  };
  turbine: {
    efficiency: number;
    designFlowM3s: number;
    maxSafeFlowM3s: number;
    designSpeedRpm: number;
    dynamics: TurbineDynamics;
  };
  generator: {
    efficiency: number;
    ratedPowerKw: number;
  };
  fluid: {
    densityKgM3: number;
    gravityMs2: number;
  };
};

/** Engine-facing hydro-plant document (camelCase JSON). */
export type HydroPlantJson = {
  schemaVersion: number;
  kind: "hydro-plant";
  id: string;
  label?: string;
  stream: { availableFlowM3s: number };
  penstock: {
    grossHeadM: number;
    lengthM: number;
    diameterM: number;
    frictionFactor: number;
    minorLossCoefficient: number;
  };
  turbine: {
    efficiency: number;
    designFlowM3s: number;
    maxSafeFlowM3s?: number;
    designSpeedRpm: number;
    dynamics: TurbineDynamics;
  };
  generator: {
    efficiency: number;
    ratedPowerKw: number;
  };
  fluid: {
    densityKgM3: number;
    gravityMs2: number;
  };
};

export function defaultPlantParams(): PlantParams {
  // Q=40 L/s, D≈18 cm → ~1.5 m/s (see designDefaults.suggestedDiameterM).
  const q = 0.04;
  const d = 0.18;
  const eta = 0.7;
  return {
    id: "lab-plant",
    label: "Lab plant",
    stream: { availableFlowM3s: q },
    penstock: {
      diameterM: d,
      frictionFactor: 0.02,
      baseMinorLossCoefficient: 0.5,
      overrideMinorLoss: false,
      overrideMinorLossCoefficient: 0.5,
      overrideHead: false,
      overrideGrossHeadM: 25,
      overrideLength: false,
      overrideLengthM: 180,
    },
    turbine: {
      efficiency: eta,
      designFlowM3s: q,
      maxSafeFlowM3s: q * 10,
      designSpeedRpm: 1000,
      dynamics: {
        speedRampUpS: 20,
        speedRampDownS: 25,
        powerRampUpS: 20,
        powerRampDownS: 25,
      },
    },
    generator: {
      efficiency: 1,
      ratedPowerKw: 1e9,
    },
    fluid: {
      densityKgM3: 1000,
      gravityMs2: 9.80665,
    },
  };
}

export function defaultOperator(): OperatorInputs {
  return {
    gateOpening: 1,
    debrisClogFraction: 0,
    leakageFraction: 0,
    online: true,
  };
}

export function clonePlantParams(p: PlantParams): PlantParams {
  return structuredClone(p);
}
