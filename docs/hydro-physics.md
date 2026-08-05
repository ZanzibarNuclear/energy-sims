# Hydro physics

**Status:** Implemented in `energy-sim-core` (PR2).

## Core equation

\[
P_\mathrm{hydraulic} = \rho\, g\, Q\, H_\mathrm{net}
\]
\[
P_\mathrm{electrical} = \eta_\mathrm{turbine}\,\eta_\mathrm{generator}\, P_\mathrm{hydraulic}
\]
\[
H_\mathrm{net} = \max\bigl(0,\, H_\mathrm{gross} - H_\mathrm{loss}\bigr)
\]

- \(P\) in watts before conversion to kW for telemetry.
- \(Q\) is **effective turbine flow** after gate, debris capture reduction, leakage, and safe-flow cap.
- Electrical power is clipped to `generator.ratedPowerKw` when above nameplate.

## Units and defaults

| Symbol | Quantity | Default |
| --- | --- | --- |
| \(\rho\) | Fluid density | \(1000\,\mathrm{kg/m^3}\) (water) |
| \(g\) | Gravity | \(9.80665\,\mathrm{m/s^2}\) |
| \(H\) | Head | meters |
| \(Q\) | Volumetric flow | \(\mathrm{m^3/s}\) |
| \(P\) | Power (API) | kW |

Override \(\rho\) and \(g\) under `fluid` in plant JSON.

## Plant JSON

See `fixtures/plants/clearwater-diversion.json` for a complete example (Clearwater Diversion on Clearwater Run). Required fields:

| Path | Role |
| --- | --- |
| `schemaVersion` | Must be ≥ 1 |
| `kind` | `"hydro-plant"` |
| `id` | Plant instance id |
| `stream.availableFlowM3s` | Stream / intake availability |
| `penstock.grossHeadM` | Elevation drop intake → turbine |
| `penstock.lengthM` / `diameterM` | Geometry for losses |
| `penstock.frictionFactor` | Darcy \(f\) (default 0.02) |
| `penstock.minorLossCoefficient` | Combined \(K\) (default 0.5) |
| `turbine.efficiency` | \(\eta_t \in [0,1]\) |
| `turbine.designFlowM3s` | Design operating flow |
| `turbine.maxSafeFlowM3s` | Optional cap |
| `turbine.designSpeedRpm` | Steady speed target (default 1000) |
| `generator.efficiency` | \(\eta_g \in [0,1]\) |
| `generator.ratedPowerKw` | Nameplate electrical cap |

Optional `packageId` / `catalogRef`-style strings may appear on plant or components; they are **ignored** until a catalog resolver exists.

## Head-loss catalog

Mean velocity \(v = Q / A\), \(A = \pi (D/2)^2\), velocity head \(v^2/(2g)\).

| Cause | Formula / mapping |
| --- | --- |
| **Pipe friction** | \(h_f = f \cdot (L/D) \cdot v^2/(2g)\) |
| **Minor (entrance, bends, fittings)** | \(h_m = K \cdot v^2/(2g)\) |
| **Debris / screen clog** | Extra \(K_\mathrm{debris} = 10 \cdot c\) where \(c\) is clog fraction; also reduces captured flow by \(0.5\,c\) |
| **Leakage** | Flow reduction: \(Q \leftarrow Q(1 - \ell)\), not head |
| **Elevation only** | Set \(f = 0\), \(K = 0\) → \(H_\mathrm{net} = H_\mathrm{gross}\) |

Ideal teaching plant: `fixtures/plants/ideal-teaching.json`.

## Operator inputs

Used by `evaluate_plant_with_inputs`:

| Field | Meaning |
| --- | --- |
| `gateOpening` | Admission 0–1; scales diverted flow |
| `debrisClogFraction` | 0–1; head loss + capture reduction |
| `leakageFraction` | 0–1; flow lost before turbine |
| `online` | When false, power delivered is 0 |

## Library entry points

```rust
use energy_sim_core::{evaluate_plant, evaluate_plant_with_inputs, HydroPlantConfig, OperatorInputs};

let plant = HydroPlantConfig::from_json(&json)?;
let eval = evaluate_plant(&plant)?;
// eval.electrical_power_kw, eval.net_head_m, eval.flow_m3s, ...
```

## Energy over time

Energy is the time-integral of **actual** electrical power. Steady segments use this evaluation directly; when the runtime ramps, integration uses ramped power, not instantaneous jumps to the target.

## Dynamics (ramp-up / ramp-down)

Plant config `turbine.dynamics` sets **time to complete** an S-curve transition (smoothstep) from the current actual to a new target:

| Field | Default | Meaning |
| --- | --- | --- |
| `speedRampUpS` / `powerRampUpS` | 20 | Seconds to reach a higher target |
| `speedRampDownS` / `powerRampDownS` | 25 | Seconds to reach a lower target |

At \(t = t_\mathrm{start} + \mathrm{duration}\), actual equals target exactly. Mid-ramp target changes restart a new segment from the current actual.

See [design.md](design.md) for architecture and key decisions.
