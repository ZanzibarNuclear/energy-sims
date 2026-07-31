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

function joinFactorsHtml(parts: string[]): string {
  if (parts.includes("−") && parts.length === 3) {
    return `${factorHtml(parts[0]!)} − ${factorHtml(parts[2]!)}`;
  }
  return parts.map(factorHtml).join(" · ");
}
</script>

<template>
  <section v-if="breakdown" class="equation" aria-label="Power equation">
    <h3>Steady power</h3>

    <div v-for="step in breakdown.steps" :key="step.id" class="step">
      <p
        class="line symbol"
        v-html="`${lhsHtml(step.lhs)} = ${joinFactorsHtml(step.factorsSymbol)}`"
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

    <div class="loss-card">
      <h4>Head loss breakdown</h4>
      <p class="loss-note">
        Losses scale with velocity head v²/(2g). Bends matter in two ways: they lengthen the pipe
        (more friction) and add turn loss K (extra minor head). At low velocity that second part is
        often only centimeters of head.
      </p>
      <dl>
        <div>
          <dt>Pipe length L</dt>
          <dd>{{ breakdown.pipeLengthM.toFixed(1) }} m</dd>
        </div>
        <div>
          <dt>Velocity</dt>
          <dd>{{ breakdown.losses.velocityMs.toFixed(2) }} m/s</dd>
        </div>
        <div>
          <dt>Friction h<sub>f</sub></dt>
          <dd>{{ breakdown.losses.frictionM.toFixed(2) }} m</dd>
        </div>
        <div>
          <dt>Entrance (K = {{ breakdown.losses.baseMinorK.toFixed(2) }})</dt>
          <dd>{{ breakdown.losses.entranceM.toFixed(2) }} m</dd>
        </div>
        <div>
          <dt>Bends (K = {{ breakdown.losses.bendMinorK.toFixed(2) }})</dt>
          <dd>{{ breakdown.losses.bendM.toFixed(2) }} m</dd>
        </div>
        <div v-if="breakdown.losses.debrisM > 0.005">
          <dt>Debris</dt>
          <dd>{{ breakdown.losses.debrisM.toFixed(2) }} m</dd>
        </div>
        <div class="total">
          <dt>H<sub>loss</sub></dt>
          <dd>{{ breakdown.losses.totalM.toFixed(2) }} m</dd>
        </div>
      </dl>
      <p v-if="breakdown.losses.bendMinorK < 0.01" class="loss-note">
        No significant bend angle yet. Drag bends off the straight line so the turn is sharper;
        K and path length will both rise.
      </p>
      <p v-if="breakdown.headStarved" class="starved" role="status">
        H<sub>loss</sub> ({{ breakdown.losses.totalM.toFixed(1) }} m) is greater than
        H<sub>gross</sub> ({{ breakdown.grossHeadM.toFixed(1) }} m). Increase diameter, reduce
        intake flow, shorten the run, or ease bends so the penstock can pass the flow.
      </p>
    </div>

    <div class="totals">
      <div>
        <span class="k">Ideal (no losses)</span>
        <span class="v">{{ breakdown.idealElectricalKw.toFixed(1) }} kW</span>
      </div>
      <div>
        <span class="k">With losses</span>
        <span class="v">{{ breakdown.withLossesKw.toFixed(1) }} kW</span>
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

h4 {
  margin: 0 0 0.35rem;
  font-size: 0.78rem;
  font-weight: 650;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--muted-fg);
}

.step {
  margin-bottom: 0.75rem;
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

.loss-card {
  margin: 0.25rem 0 0.85rem;
  padding: 0.65rem 0.75rem;
  border-radius: 8px;
  background: var(--bg);
  border: 1px solid var(--border);
}

.loss-note {
  margin: 0 0 0.5rem;
  font-size: 0.78rem;
  line-height: 1.4;
  color: var(--muted-fg);
}

.loss-card dl {
  margin: 0;
  display: grid;
  gap: 0.25rem;
}

.loss-card dl div {
  display: flex;
  justify-content: space-between;
  font-size: 0.85rem;
}

.loss-card dt {
  color: var(--muted-fg);
}

.loss-card dd {
  margin: 0;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

.loss-card .total {
  margin-top: 0.2rem;
  padding-top: 0.3rem;
  border-top: 1px solid var(--border);
}

.starved {
  margin: 0.55rem 0 0;
  font-size: 0.82rem;
  line-height: 1.4;
  color: var(--fg);
  padding: 0.45rem 0.55rem;
  border-radius: 6px;
  background: color-mix(in srgb, #c9a227 18%, transparent);
  border: 1px solid color-mix(in srgb, #c9a227 40%, var(--border));
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
