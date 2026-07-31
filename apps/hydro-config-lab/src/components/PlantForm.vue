<script setup lang="ts">
import { computed } from "vue";
import {
  DIAMETER_MAX_M,
  DIAMETER_MIN_M,
  DIAMETER_STEP_M,
  ETA_MAX,
  ETA_MIN,
  ETA_STEP,
  FIXED_BASE_MINOR_K,
  FIXED_FRICTION,
  FLOW_MAX_M3S,
  FLOW_MIN_M3S,
  FLOW_STEP_M3S,
  clamp,
  lsToM3s,
  m3sToLs,
  meanVelocityMs,
  suggestedDiameterM,
  TARGET_VELOCITY_MS,
} from "../lib/designDefaults";
import type { OperatorInputs, PlantParams } from "../lib/plantParams";

const props = defineProps<{
  params: PlantParams;
  operator: OperatorInputs;
}>();

const emit = defineEmits<{
  "update:params": [PlantParams];
  "update:operator": [OperatorInputs];
}>();

function plainClone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v)) as T;
}

/** Overall η = η_t × η_g; we keep η_g fixed and fold design into η_t for simplicity. */
const overallEta = computed(() =>
  clamp(props.params.turbine.efficiency * props.params.generator.efficiency, ETA_MIN, ETA_MAX),
);

const flowLs = computed(() => m3sToLs(props.params.stream.availableFlowM3s));
const diameterCm = computed(() => props.params.penstock.diameterM * 100);
const velocity = computed(() =>
  meanVelocityMs(props.params.stream.availableFlowM3s, props.params.penstock.diameterM),
);
const suggestedD = computed(() =>
  suggestedDiameterM(props.params.stream.availableFlowM3s, TARGET_VELOCITY_MS),
);
const suggestedCm = computed(() => suggestedD.value * 100);

function applyFixedHidden(next: PlantParams) {
  next.penstock.frictionFactor = FIXED_FRICTION;
  next.penstock.baseMinorLossCoefficient = FIXED_BASE_MINOR_K;
  next.penstock.overrideHead = false;
  next.penstock.overrideLength = false;
  next.penstock.overrideMinorLoss = false;
  next.generator.efficiency = 1;
  next.generator.ratedPowerKw = 1e9;
  next.turbine.designFlowM3s = next.stream.availableFlowM3s;
  next.turbine.maxSafeFlowM3s = next.stream.availableFlowM3s * 10;
  // Gate full open for design calculations; Run can still change operator later if needed.
  return next;
}

function setFlowLs(raw: string) {
  const ls = Number(raw);
  if (!Number.isFinite(ls)) return;
  const q = clamp(lsToM3s(ls), FLOW_MIN_M3S, FLOW_MAX_M3S);
  const next = plainClone(props.params);
  next.stream.availableFlowM3s = q;
  next.turbine.designFlowM3s = q;
  emit("update:params", applyFixedHidden(next));
  emit("update:operator", {
    gateOpening: 1,
    debrisClogFraction: 0,
    leakageFraction: 0,
    online: true,
  });
}

function setDiameterCm(raw: string) {
  const cm = Number(raw);
  if (!Number.isFinite(cm)) return;
  const d = clamp(cm / 100, DIAMETER_MIN_M, DIAMETER_MAX_M);
  const next = plainClone(props.params);
  next.penstock.diameterM = d;
  emit("update:params", applyFixedHidden(next));
}

function setOverallEta(raw: string) {
  const eta = Number(raw);
  if (!Number.isFinite(eta)) return;
  const next = plainClone(props.params);
  // Store all of overall η on the turbine; generator η = 1 so product is overall.
  next.turbine.efficiency = clamp(eta, ETA_MIN, ETA_MAX);
  next.generator.efficiency = 1;
  emit("update:params", applyFixedHidden(next));
}

function applySuggestedDiameter() {
  const next = plainClone(props.params);
  next.penstock.diameterM = suggestedD.value;
  emit("update:params", applyFixedHidden(next));
}

const velocityHint = computed(() => {
  const v = velocity.value;
  if (v < 0.8) return "Slow flow — large pipe for this Q (low friction, bulkier).";
  if (v > 3) return "Fast flow — small pipe or high Q (head loss climbs quickly).";
  return "Typical micro-hydro range (~1–2.5 m/s).";
});
</script>

<template>
  <div class="design">
    <header class="head">
      <h2>Design choices</h2>
      <p>
        Layout already set head, pipe length, and bend losses. Here you only size the intake flow
        and penstock, plus overall plant efficiency.
      </p>
    </header>

    <div class="fields">
      <label class="field">
        <span>Intake flow (L/s)</span>
        <input
          type="number"
          :min="m3sToLs(FLOW_MIN_M3S)"
          :max="m3sToLs(FLOW_MAX_M3S)"
          :step="m3sToLs(FLOW_STEP_M3S)"
          :value="flowLs.toFixed(1)"
          @change="setFlowLs(($event.target as HTMLInputElement).value)"
        />
        <span class="hint">{{ FLOW_MIN_M3S * 1000 }}–{{ FLOW_MAX_M3S * 1000 }} L/s diverted into the intake</span>
      </label>

      <label class="field">
        <span>Penstock diameter (cm)</span>
        <input
          type="number"
          :min="DIAMETER_MIN_M * 100"
          :max="DIAMETER_MAX_M * 100"
          :step="DIAMETER_STEP_M * 100"
          :value="diameterCm.toFixed(0)"
          @change="setDiameterCm(($event.target as HTMLInputElement).value)"
        />
        <span class="hint">
          Mean velocity {{ velocity.toFixed(2) }} m/s.
          {{ velocityHint }}
        </span>
      </label>

      <div class="suggest">
        <p>
          For ~{{ TARGET_VELOCITY_MS }} m/s at this flow, try
          <strong>{{ suggestedCm.toFixed(0) }} cm</strong> diameter.
        </p>
        <button type="button" class="btn" @click="applySuggestedDiameter">Use suggested diameter</button>
      </div>

      <label class="field">
        <span>Overall efficiency η (turbine × generator)</span>
        <input
          type="number"
          :min="ETA_MIN"
          :max="ETA_MAX"
          :step="ETA_STEP"
          :value="overallEta.toFixed(2)"
          @change="setOverallEta(($event.target as HTMLInputElement).value)"
        />
        <span class="hint">Teaching default ~0.7. Higher = better machines, not more water.</span>
      </label>
    </div>

    <p class="fixed-note">
      Fixed for now (not design knobs): pipe friction f = {{ FIXED_FRICTION }}, entrance K =
      {{ FIXED_BASE_MINOR_K }}, full gate, no debris or leakage. Bend losses still come from Layout.
    </p>
  </div>
</template>

<style scoped>
.design {
  padding: 1rem 1.1rem 1.15rem;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--panel);
}

.head h2 {
  margin: 0 0 0.35rem;
  font-size: 1.05rem;
  font-weight: 650;
}

.head p {
  margin: 0 0 0.9rem;
  font-size: 0.88rem;
  line-height: 1.45;
  color: var(--muted-fg);
  max-width: 40rem;
}

.fields {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.85rem 1.25rem;
  align-items: start;
}

@media (max-width: 640px) {
  .fields {
    grid-template-columns: 1fr;
  }
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.8rem;
  color: var(--muted-fg);
}

.field span:first-child {
  font-weight: 600;
  color: var(--fg);
  font-size: 0.88rem;
}

.field input {
  font: inherit;
  font-size: 1rem;
  color: var(--fg);
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.45rem 0.55rem;
  max-width: 12rem;
}

.hint {
  font-size: 0.78rem;
  line-height: 1.35;
  color: var(--muted-fg);
}

.suggest {
  grid-column: 1 / -1;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.65rem 1rem;
  padding: 0.65rem 0.75rem;
  border-radius: 8px;
  background: var(--bg);
  border: 1px solid var(--border);
}

.suggest p {
  margin: 0;
  font-size: 0.88rem;
  color: var(--fg);
}

.btn {
  font: inherit;
  font-size: 0.85rem;
  font-weight: 600;
  padding: 0.4rem 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--accent);
  background: var(--accent);
  color: #fff;
  cursor: pointer;
}

.btn:hover {
  filter: brightness(1.06);
}

.fixed-note {
  margin: 1rem 0 0;
  font-size: 0.78rem;
  line-height: 1.4;
  color: var(--muted-fg);
}
</style>
