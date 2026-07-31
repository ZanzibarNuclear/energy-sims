import { describe, expect, it } from "vitest";
import {
  constrainBendPointDetailed,
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

  it("reports uphill when raising a bend above its upstream neighbor", () => {
    const site = defaultSite();
    site.bends = [{ sM: 90, zM: 25 }];
    const r = constrainBendPointDetailed(site, 0, { sM: 90, zM: 80 });
    expect(r.reason).toBe("uphill");
    expect(r.message.toLowerCase()).toContain("downhill");
    expect(r.point.zM).toBeLessThanOrEqual(site.intake.zM);
  });

  it("reports below_turbine when lowering a bend under the turbine", () => {
    const site = defaultSite();
    site.bends = [{ sM: 90, zM: 25 }];
    const r = constrainBendPointDetailed(site, 0, { sM: 90, zM: -20 });
    expect(r.reason).toBe("below_turbine");
    expect(r.message.toLowerCase()).toContain("turbine");
    expect(r.point.zM).toBeGreaterThanOrEqual(site.turbine.zM);
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
