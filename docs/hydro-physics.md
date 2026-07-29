# Hydro physics

**Status:** Stub — filled in by PR2 (`feat(energy-sim): hydro power equation and plant config JSON`).

## Core equation (Stage 1)

\[
P_\mathrm{hydraulic} = \rho\, g\, Q\, H_\mathrm{net}
\]
\[
P_\mathrm{electrical} = \eta_\mathrm{turbine}\,\eta_\mathrm{generator}\, P_\mathrm{hydraulic}
\]
\[
H_\mathrm{net} = \max\bigl(0,\, H_\mathrm{gross} - H_\mathrm{loss}\bigr)
\]

## Planned contents

- Units and defaults (\(\rho\), \(g\), SI)
- Gross head, flow, penstock geometry
- Loss catalog (friction / minor losses) with nominal coefficients
- Turbine and generator efficiencies
- Rated / design operating point and caps
- JSON plant config field reference

See [design.md](design.md) for architecture and key decisions.
