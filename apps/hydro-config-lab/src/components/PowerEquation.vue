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

/** Pretty LHS with subscripts for display. */
function lhsHtml(lhs: string): string {
  const map: Record<string, string> = {
    Hnet: "H<sub>net</sub>",
    Q: "Q",
    Ph: "P<sub>h</sub>",
    Pe: "P<sub>e</sub>",
  };
  return map[lhs] ?? lhs;
}

function factorHtml(tok: string): string {
  const map: Record<string, string> = {
    Hgross: "H<sub>gross</sub>",
    Hloss: "H<sub>loss</sub>",
    Hnet: "H<sub>net</sub>",
    Qintake: "Q<sub>intake</sub>",
    "ηt": "η<sub>t</sub>",
    "ηg": "η<sub>g</sub>",
    Ph: "P<sub>h</sub>",
    "ρ": "ρ",
    g: "g",
    Q: "Q",
    gate: "gate",
    "(1−½·debris)": "(1 − ½·debris)",
    "(1−leak)": "(1 − leak)",
    "−": "−",
  };
  return map[tok] ?? tok;
}

function joinFactors(parts: string[], asHtml: boolean): string {
  // Hnet step uses − as a binary operator between two terms, not ·
  if (parts.includes("−") && parts.length === 3) {
    const [a, op, b] = parts;
    if (asHtml) {
      return `${factorHtml(a!)} ${op} ${factorHtml(b!)}`;
    }
    return `${a} ${op} ${b}`;
  }
  if (asHtml) {
    return parts.map(factorHtml).join(" · ");
  }
  return parts.join(" · ");
}
</script>

<template>
  <section v-if="breakdown" class="equation" aria-label="Power equation">
    <h3>Steady power</h3>

    <div v-for="step in breakdown.steps" :key="step.id" class="step">
      <p
        class="line symbol"
        v-html="`${lhsHtml(step.lhs)} = ${joinFactors(step.factorsSymbol, true)}`"
      />
      <p class="line numeric">
        <span v-html="lhsHtml(step.lhs)" />
        =
        {{
          step.factorsNumeric.includes("−") && step.factorsNumeric.length === 3
            ? `${step.factorsNumeric[0]} − ${step.factorsNumeric[2]}`
            : step.factorsNumeric.join(" · ")
        }}
        =
        <strong>{{ step.result }}</strong>
      </p>
    </div>

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
        <span class="k">After nameplate</span>
        <span class="v">{{ breakdown.electricalKw.toFixed(1) }} kW</span>
      </div>
    </div>
  </section>
  <section v-else class="equation muted">
    <h3>Steady power</h3>
    <p>Finish Layout so head and length are defined.</p>
  </section>
</template>

<style scoped>
.equation {
  margin-top: 0.75rem;
  padding: 0.85rem 1rem;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--panel);
}

.equation.muted {
  color: var(--muted-fg);
}

h3 {
  margin: 0 0 0.65rem;
  font-size: 0.95rem;
  font-weight: 650;
}

.step {
  margin-bottom: 0.75rem;
}

.step:last-of-type {
  margin-bottom: 0.85rem;
}

.line {
  margin: 0;
  line-height: 1.45;
  font-size: 0.95rem;
}

.symbol {
  font-weight: 650;
  margin-bottom: 0.2rem;
}

.numeric {
  color: var(--muted-fg);
  font-variant-numeric: tabular-nums;
  font-size: 0.88rem;
}

.numeric strong {
  color: var(--fg);
  font-weight: 700;
}

.totals {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem 1.1rem;
  padding-top: 0.55rem;
  border-top: 1px solid var(--border);
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
