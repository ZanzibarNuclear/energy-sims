/**
 * Minimal node test for presentation helpers (no build step).
 * Run: node --test clients/js/energySimPresent.test.mjs
 */
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  presentGrid,
  presentHydro,
  presentSnapshot,
} from "./energySimPresent.js";

const sample = {
  simTimeS: 10,
  phase: "running",
  plantId: "clearwater-diversion",
  flowM3s: 0.05,
  grossHeadM: 25,
  netHeadM: 24,
  headLossM: 1,
  hydraulicPowerKw: 10,
  electricalPowerKw: 7.5,
  targetElectricalPowerKw: 8,
  turbineSpeedRpm: 900,
  targetTurbineSpeedRpm: 1000,
  energyGeneratedKwh: 0.02,
  availableGenerationKw: 7.5,
  totalLoadKw: 3.5,
  marginKw: 4,
  busEnergized: true,
  gridStatus: "surplus",
  loads: [
    {
      id: "ev-charge.port-1",
      label: "EV charge port",
      ratingW: 3500,
      priority: "deferrable",
      drawing: true,
    },
    {
      id: "lighting.main",
      label: "Main lights",
      ratingW: 400,
      priority: "normal",
      drawing: false,
    },
  ],
  warnings: [],
};

describe("presentHydro", () => {
  it("maps sensor fields", () => {
    const h = presentHydro(sample);
    assert.equal(h.plantId, "clearwater-diversion");
    assert.equal(h.electricalPowerKw, 7.5);
    assert.equal(h.turbineSpeedRpm, 900);
  });
});

describe("presentGrid", () => {
  it("includes load table and light level", () => {
    const g = presentGrid(sample);
    assert.equal(g.loads.length, 2);
    assert.equal(g.loads[0].drawing, true);
    assert.equal(g.lightLevel, 1.0);
    assert.equal(g.brownout, false);
  });

  it("dims on brownout", () => {
    const g = presentGrid({ ...sample, gridStatus: "brownout", marginKw: -1 });
    assert.equal(g.brownout, true);
    assert.equal(g.lightLevel, 0.4);
  });
});

describe("presentSnapshot", () => {
  it("nests hydro and grid", () => {
    const v = presentSnapshot(sample);
    assert.ok(v.hydro);
    assert.ok(v.grid);
    assert.equal(v.grid.loads.length, 2);
    assert.equal(v.electricalPowerKw, 7.5);
  });
});
