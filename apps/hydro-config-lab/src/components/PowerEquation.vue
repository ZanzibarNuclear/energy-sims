<script setup lang="ts">
import { computed } from "vue";
import type { DerivedGeometry } from "../lib/compileSite";
import { computePowerBreakdown } from "../lib/powerEquation";
import type { OperatorInputs, PlantParams } from "../lib/plantParams";

const props = defineProps<{
  params: PlantParams;
  operator: OperatorInputs;
  derived: DerivedGeometry | null;
}>();

const breakdown = computed(() =>
  computePowerBreakdown(props.params, props.operator, props.derived),
);
</script>

<template>
  <section v-if="breakdown" class="equation" aria-label="Power equation">
    <h3>Steady power from the configuration</h3>
    <p class="lead">
      \(P = \eta_t\,\eta_g\,\rho\, g\, Q\, H_\mathrm{net}\) with
      \(H_\mathrm{net} = H_\mathrm{gross} - H_\mathrm{loss}\). Numbers below use this layout and
      equipment (same formulas as the engine’s steady evaluation).
    </p>
    <ul class="lines">
      <li v-for="(line, i) in breakdown.lines" :key="i">
        <code>{{ line }}</code>
      </li>
    </ul>
    <div class="totals">
      <div>
        <span class="k">Ideal (no losses)</span>
        <span class="v">{{ breakdown.idealElectricalKw.toFixed(3) }} kW</span>
      </div>
      <div>
        <span class="k">With losses (uncapped)</span>
        <span class="v">{{ breakdown.uncappedElectricalKw.toFixed(3) }} kW</span>
      </div>
      <div>
        <span class="k">After nameplate cap</span>
        <span class="v">{{ breakdown.electricalKw.toFixed(3) }} kW</span>
      </div>
    </div>
    <p class="foot">
      Bends raise minor-loss K (from Layout turn angles), which increases H_loss and lowers
      H_net. The Run tab ramps toward this steady target rather than jumping instantly.
    </p>
  </section>
  <section v-else class="equation muted">
    <h3>Steady power from the configuration</h3>
    <p>Finish the Layout tab so head and pipe length are defined.</p>
  </section>
</template>

<style scoped>
.equation {
  margin-top: 0.85rem;
  padding: 0.85rem 1rem;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--panel);
}

.equation.muted {
  color: var(--muted-fg);
}

h3 {
  margin: 0 0 0.4rem;
  font-size: 0.95rem;
  font-weight: 650;
}

.lead {
  margin: 0 0 0.55rem;
  font-size: 0.85rem;
  line-height: 1.45;
  color: var(--muted-fg);
}

.lines {
  margin: 0 0 0.65rem;
  padding-left: 1.1rem;
}

.lines code {
  font-size: 0.78rem;
  line-height: 1.5;
  background: var(--code-bg);
  padding: 0.15rem 0.35rem;
  border-radius: 4px;
  display: inline-block;
  margin: 0.15rem 0;
  white-space: pre-wrap;
  word-break: break-word;
}

.totals {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem 1.25rem;
  margin-bottom: 0.5rem;
}

.totals .k {
  display: block;
  font-size: 0.68rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--muted-fg);
}

.totals .v {
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  font-size: 0.95rem;
}

.foot {
  margin: 0;
  font-size: 0.78rem;
  line-height: 1.4;
  color: var(--muted-fg);
}
</style>
