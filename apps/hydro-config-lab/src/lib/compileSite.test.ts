import { describe, expect, it } from "vitest";
import {
  bendLossK,
  compileSite,
  deriveGeometry,
  grossHeadM,
  K_BEND_90,
  pathLengthM,
  segmentLengthM,
  turnAngleRad,
} from "./compileSite";
import { defaultPlantParams } from "./plantParams";
import type { Site } from "./site";

describe("segment and path length", () => {
  it("computes 3-4-5 segment in s–z plane", () => {
    expect(segmentLengthM({ sM: 0, zM: 0 }, { sM: 3, zM: 4 })).toBeCloseTo(5, 10);
  });

  it("sums path length", () => {
    const L = pathLengthM([
      { sM: 0, zM: 10 },
      { sM: 0, zM: 0 },
      { sM: 10, zM: 0 },
    ]);
    expect(L).toBeCloseTo(20, 10);
  });
});

describe("gross head", () => {
  it("is elevation drop intake → turbine", () => {
    expect(grossHeadM({ sM: 0, zM: 100 }, { sM: 50, zM: 75 })).toBe(25);
  });

  it("clamps negative (uphill) to zero", () => {
    expect(grossHeadM({ sM: 0, zM: 10 }, { sM: 20, zM: 30 })).toBe(0);
  });
});

describe("bend losses", () => {
  it("straight run has zero turn angle", () => {
    const ang = turnAngleRad({ sM: 0, zM: 10 }, { sM: 10, zM: 5 }, { sM: 20, zM: 0 });
    // nearly colinear downhill
    expect(ang).toBeLessThan(0.01);
  });

  it("right-angle path contributes K_BEND_90", () => {
    const ang = turnAngleRad({ sM: 0, zM: 0 }, { sM: 10, zM: 0 }, { sM: 10, zM: -10 });
    expect(ang).toBeCloseTo(Math.PI / 2, 6);
    expect(bendLossK(ang)).toBeCloseTo(K_BEND_90, 10);
  });

  it("extra bend increases minor K vs straight", () => {
    const straight: Site = {
      intake: { sM: 0, zM: 25 },
      bends: [],
      turbine: { sM: 100, zM: 0 },
    };
    const bent: Site = {
      intake: { sM: 0, zM: 25 },
      bends: [{ sM: 50, zM: 25 }],
      turbine: { sM: 100, zM: 0 },
    };
    const d0 = deriveGeometry(straight, 0.5)!;
    const d1 = deriveGeometry(bent, 0.5)!;
    expect(d1.bendMinorK).toBeGreaterThan(d0.bendMinorK);
    expect(d1.minorLossCoefficient).toBeGreaterThan(d0.minorLossCoefficient);
    expect(d1.grossHeadM).toBeCloseTo(d0.grossHeadM, 10);
  });
});

describe("compileSite", () => {
  const site: Site = {
    intake: { sM: 0, zM: 100 },
    bends: [],
    turbine: { sM: 120, zM: 75 },
  };

  it("fails when intake and turbine coincide (zero length)", () => {
    const r = compileSite(
      { intake: { sM: 0, zM: 10 }, bends: [], turbine: { sM: 0, zM: 10 } },
      defaultPlantParams(),
    );
    expect(r.ok).toBe(false);
  });

  it("emits plant JSON with derived head and length", () => {
    const r = compileSite(site, defaultPlantParams());
    expect(r.ok).toBe(true);
    if (!r.ok) return;
    expect(r.plant.kind).toBe("hydro-plant");
    expect(r.plant.penstock.grossHeadM).toBeCloseTo(25, 10);
    expect(r.plant.penstock.lengthM).toBeCloseTo(Math.hypot(120, 25), 8);
    expect(r.plant.penstock.minorLossCoefficient).toBeCloseTo(0.5, 8);
  });

  it("ignores head override — layout geometry wins", () => {
    const p = defaultPlantParams();
    p.penstock.overrideHead = true;
    p.penstock.overrideGrossHeadM = 40;
    const r = compileSite(site, p);
    expect(r.ok).toBe(true);
    if (!r.ok) return;
    expect(r.plant.penstock.grossHeadM).toBeCloseTo(25, 10);
    expect(r.usedOverrides.head).toBe(false);
  });
});
