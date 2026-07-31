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

    <p class="headline">
      <strong>P<sub>e</sub> = η<sub>t</sub> · η<sub>g</sub> · ρ · g · Q · H<sub>net</sub></strong>
    </p>
    <p class="headline">
      <strong>H<sub>net</sub> = H<sub>gross</sub> − H<sub>loss</sub></strong>
    </p>

    <ol class="steps">
      <li v-for="(step, i) in breakdown.steps" :key="i" class="step">
        <div class="symbol">{{ step.symbol }}</div>
        <div class="numeric">
          <code>{{ step.numeric }}</code>
        </div>
      </li>
    </ol>

    <div class="totals">
      <div>
        <span class="k">Ideal (no losses)</span>
        <span class="v">{{ breakdown.idealElectricalKw.toFixed(1) }} kW</span>
      </div>
      <div>
        <span class="k">With losses (uncapped)</span>
        <span class="v">{{ breakdown.uncappedElectricalKw.toFixed(1) }} kW</span>
      </div>
      <div>
        <span class="k">After nameplate cap</span>
        <span class="v">{{ breakdown.electricalKw.toFixed(1) }} kW</span>
      </div>
    </div>
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
  margin: 0 0 0.55rem;
  font-size: 0.95rem;
  font-weight: 650;
}

.headline {
  margin: 0 0 0.25rem;
  font-size: 0.95rem;
  line-height: 1.4;
}

.steps {
  margin: 0.75rem 0 0.85rem;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
}

.step {
  padding: 0.5rem 0.65rem;
  border-radius: 8px;
  background: var(--bg);
  border: 1px solid var(--border);
}

.symbol {
  font-size: 0.9rem;
  font-weight: 650;
  font-family: "IBM Plex Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
  margin-bottom: 0.3rem;
  color: var(--fg);
}

.numeric code {
  font-size: 0.82rem;
  line-height: 1.45;
  background: var(--code-bg);
  padding: 0.2rem 0.4rem;
  border-radius: 4px;
  display: inline-block;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--muted-fg);
}

.totals {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem 1.25rem;
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
</style>
