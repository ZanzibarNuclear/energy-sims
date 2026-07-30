import { describe, expect, it } from "vitest";
import {
  constrainBendPoint,
  defaultSite,
  moveBend,
  moveIntake,
  validatePenstock,
} from "./site";

describe("penstock gravity rules", () => {
  it("flags an uphill bend", () => {
    const site = defaultSite();
    site.bends = [{ sM: 90, zM: 80 }]; // above intake at 50
    const issues = validatePenstock(site);
    expect(issues.some((i) => i.code === "above_intake" || i.code === "uphill")).toBe(true);
  });

  it("flags a point below the turbine", () => {
    const site = defaultSite();
    site.bends = [{ sM: 90, zM: -10 }];
    const issues = validatePenstock(site);
    expect(issues.some((i) => i.code === "below_turbine")).toBe(true);
  });

  it("clamps a bend so it cannot rise above the intake", () => {
    const site = defaultSite();
    site.bends = [{ sM: 90, zM: 25 }];
    const clamped = constrainBendPoint(site, 0, { sM: 90, zM: 80 });
    expect(clamped.zM).toBeLessThanOrEqual(site.intake.zM);
    expect(clamped.zM).toBeGreaterThanOrEqual(site.turbine.zM);
  });

  it("moveBend keeps downhill profile", () => {
    const site = defaultSite();
    site.bends = [{ sM: 90, zM: 25 }];
    const next = moveBend(site, 0, { sM: 90, zM: 100 });
    expect(next.bends[0]!.zM).toBeLessThanOrEqual(next.intake.zM + 1e-9);
    expect(validatePenstock(next).filter((i) => i.code === "uphill")).toHaveLength(0);
  });

  it("moveIntake cannot drop below a bend", () => {
    const site = defaultSite();
    site.bends = [{ sM: 90, zM: 30 }];
    const next = moveIntake(site, { sM: 30, zM: 10 });
    expect(next.intake.zM).toBeGreaterThanOrEqual(30);
  });
});
