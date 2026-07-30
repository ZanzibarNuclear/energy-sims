<script setup lang="ts">
import { computed, ref } from "vue";
import type { OperatorInputs, PlantParams } from "../lib/plantParams";

const props = defineProps<{
  params: PlantParams;
  operator: OperatorInputs;
}>();

const emit = defineEmits<{
  "update:params": [PlantParams];
  "update:operator": [OperatorInputs];
}>();

type SectionId = "stream" | "penstock" | "turbine" | "generator" | "operator";

const sections: {
  id: SectionId;
  label: string;
  blurb: string;
}[] = [
  { id: "stream", label: "Stream", blurb: "Available flow" },
  { id: "penstock", label: "Penstock", blurb: "Pipe size & losses" },
  { id: "turbine", label: "Turbine", blurb: "η, flow, ramps" },
  { id: "generator", label: "Generator", blurb: "η, nameplate" },
  { id: "operator", label: "Operator", blurb: "Gate, debris, online" },
];

const active = ref<SectionId>("stream");

const activeMeta = computed(() => sections.find((s) => s.id === active.value)!);

function plainClone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v)) as T;
}

function setNum(path: string[], raw: string) {
  const n = Number(raw);
  if (!Number.isFinite(n) && raw !== "") return;
  const next = plainClone(props.params);
  let cur: Record<string, unknown> = next as unknown as Record<string, unknown>;
  for (let i = 0; i < path.length - 1; i++) {
    cur = cur[path[i]!] as Record<string, unknown>;
  }
  cur[path[path.length - 1]!] = n;
  // Geometry always comes from Layout — never carry override flags.
  next.penstock.overrideHead = false;
  next.penstock.overrideLength = false;
  next.penstock.overrideMinorLoss = false;
  emit("update:params", next);
}

function setStr(path: string[], raw: string) {
  const next = plainClone(props.params);
  let cur: Record<string, unknown> = next as unknown as Record<string, unknown>;
  for (let i = 0; i < path.length - 1; i++) {
    cur = cur[path[i]!] as Record<string, unknown>;
  }
  cur[path[path.length - 1]!] = raw;
  emit("update:params", next);
}

function setOpNum(key: keyof OperatorInputs, raw: string) {
  const n = Number(raw);
  if (!Number.isFinite(n) && raw !== "") return;
  emit("update:operator", { ...props.operator, [key]: n });
}

function setOpBool(key: keyof OperatorInputs, value: boolean) {
  emit("update:operator", { ...props.operator, [key]: value });
}

function summary(id: SectionId): string {
  const p = props.params;
  const o = props.operator;
  switch (id) {
    case "stream":
      return `${p.stream.availableFlowM3s} m³/s`;
    case "penstock":
      return `Ø ${p.penstock.diameterM} m · f ${p.penstock.frictionFactor}`;
    case "turbine":
      return `η ${p.turbine.efficiency} · ${p.turbine.designFlowM3s} m³/s`;
    case "generator":
      return `η ${p.generator.efficiency} · ${p.generator.ratedPowerKw} kW`;
    case "operator":
      return o.online ? `gate ${o.gateOpening}` : "offline";
  }
}
</script>

<template>
  <div class="equipment">
    <nav class="nav" aria-label="Equipment parts">
      <button
        v-for="s in sections"
        :key="s.id"
        type="button"
        class="nav-item"
        :class="{ active: active === s.id }"
        @click="active = s.id"
      >
        <span class="nav-label">{{ s.label }}</span>
        <span class="nav-summary">{{ summary(s.id) }}</span>
      </button>
    </nav>

    <div class="detail">
      <header class="detail-head">
        <h2>{{ activeMeta.label }}</h2>
        <p>{{ activeMeta.blurb }}</p>
      </header>

      <!-- Stream -->
      <div v-if="active === 'stream'" class="fields">
        <label class="field">
          <span>Available flow (m³/s)</span>
          <input
            type="number"
            step="0.001"
            min="0"
            :value="params.stream.availableFlowM3s"
            @change="setNum(['stream', 'availableFlowM3s'], ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="field">
          <span>Config id</span>
          <input
            type="text"
            :value="params.id"
            @change="setStr(['id'], ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="field wide">
          <span>Label</span>
          <input
            type="text"
            :value="params.label"
            @change="setStr(['label'], ($event.target as HTMLInputElement).value)"
          />
        </label>
      </div>

      <!-- Penstock (no geometry overrides — head/length from Layout) -->
      <div v-else-if="active === 'penstock'" class="fields">
        <label class="field">
          <span>Diameter (m)</span>
          <input
            type="number"
            step="0.01"
            min="0.01"
            :value="params.penstock.diameterM"
            @change="setNum(['penstock', 'diameterM'], ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="field">
          <span>Friction factor f</span>
          <input
            type="number"
            step="0.001"
            min="0"
            :value="params.penstock.frictionFactor"
            @change="
              setNum(['penstock', 'frictionFactor'], ($event.target as HTMLInputElement).value)
            "
          />
        </label>
        <label class="field">
          <span>Base minor K</span>
          <input
            type="number"
            step="0.05"
            min="0"
            :value="params.penstock.baseMinorLossCoefficient"
            @change="
              setNum(
                ['penstock', 'baseMinorLossCoefficient'],
                ($event.target as HTMLInputElement).value,
              )
            "
          />
        </label>
        <p class="note">
          Head, pipe length, and bend losses come from the Layout tab. Base K is entrance/fittings
          only; bends add more automatically.
        </p>
      </div>

      <!-- Turbine -->
      <div v-else-if="active === 'turbine'" class="fields">
        <label class="field">
          <span>Efficiency η</span>
          <input
            type="number"
            step="0.01"
            min="0"
            max="1"
            :value="params.turbine.efficiency"
            @change="setNum(['turbine', 'efficiency'], ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="field">
          <span>Design flow (m³/s)</span>
          <input
            type="number"
            step="0.001"
            :value="params.turbine.designFlowM3s"
            @change="setNum(['turbine', 'designFlowM3s'], ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="field">
          <span>Max safe flow (m³/s)</span>
          <input
            type="number"
            step="0.001"
            :value="params.turbine.maxSafeFlowM3s"
            @change="
              setNum(['turbine', 'maxSafeFlowM3s'], ($event.target as HTMLInputElement).value)
            "
          />
        </label>
        <label class="field">
          <span>Design speed (rpm)</span>
          <input
            type="number"
            step="1"
            :value="params.turbine.designSpeedRpm"
            @change="
              setNum(['turbine', 'designSpeedRpm'], ($event.target as HTMLInputElement).value)
            "
          />
        </label>
        <label class="field">
          <span>Power ramp-up (s)</span>
          <input
            type="number"
            step="1"
            :value="params.turbine.dynamics.powerRampUpS"
            @change="
              setNum(
                ['turbine', 'dynamics', 'powerRampUpS'],
                ($event.target as HTMLInputElement).value,
              )
            "
          />
        </label>
        <label class="field">
          <span>Power ramp-down (s)</span>
          <input
            type="number"
            step="1"
            :value="params.turbine.dynamics.powerRampDownS"
            @change="
              setNum(
                ['turbine', 'dynamics', 'powerRampDownS'],
                ($event.target as HTMLInputElement).value,
              )
            "
          />
        </label>
      </div>

      <!-- Generator -->
      <div v-else-if="active === 'generator'" class="fields">
        <label class="field">
          <span>Efficiency η</span>
          <input
            type="number"
            step="0.01"
            min="0"
            max="1"
            :value="params.generator.efficiency"
            @change="
              setNum(['generator', 'efficiency'], ($event.target as HTMLInputElement).value)
            "
          />
        </label>
        <label class="field">
          <span>Rated power (kW)</span>
          <input
            type="number"
            step="0.1"
            :value="params.generator.ratedPowerKw"
            @change="
              setNum(['generator', 'ratedPowerKw'], ($event.target as HTMLInputElement).value)
            "
          />
        </label>
      </div>

      <!-- Operator -->
      <div v-else-if="active === 'operator'" class="fields">
        <label class="field">
          <span>Gate opening (0–1)</span>
          <input
            type="number"
            step="0.05"
            min="0"
            max="1"
            :value="operator.gateOpening"
            @change="setOpNum('gateOpening', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="field">
          <span>Debris clog (0–1)</span>
          <input
            type="number"
            step="0.05"
            min="0"
            max="1"
            :value="operator.debrisClogFraction"
            @change="setOpNum('debrisClogFraction', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="field">
          <span>Leakage (0–1)</span>
          <input
            type="number"
            step="0.05"
            min="0"
            max="1"
            :value="operator.leakageFraction"
            @change="setOpNum('leakageFraction', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="check">
          <input
            type="checkbox"
            :checked="operator.online"
            @change="setOpBool('online', ($event.target as HTMLInputElement).checked)"
          />
          Online
        </label>
      </div>
    </div>
  </div>
</template>

<style scoped>
.equipment {
  display: grid;
  grid-template-columns: minmax(10rem, 13rem) minmax(0, 1fr);
  gap: 0;
  min-height: 14rem;
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: hidden;
  background: var(--panel);
}

@media (max-width: 640px) {
  .equipment {
    grid-template-columns: 1fr;
  }
}

.nav {
  display: flex;
  flex-direction: column;
  gap: 0;
  background: var(--toolbar);
  border-right: 1px solid var(--border);
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.1rem;
  padding: 0.65rem 0.75rem;
  border: none;
  border-bottom: 1px solid var(--border);
  background: transparent;
  color: var(--fg);
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.nav-item:hover {
  background: color-mix(in srgb, var(--panel) 60%, transparent);
}

.nav-item.active {
  background: var(--panel);
  box-shadow: inset 3px 0 0 var(--accent);
}

.nav-label {
  font-size: 0.9rem;
  font-weight: 650;
}

.nav-summary {
  font-size: 0.72rem;
  color: var(--muted-fg);
  font-variant-numeric: tabular-nums;
}

.detail {
  padding: 0.85rem 1rem 1rem;
}

.detail-head {
  margin-bottom: 0.75rem;
}

.detail-head h2 {
  margin: 0;
  font-size: 1rem;
  font-weight: 650;
}

.detail-head p {
  margin: 0.15rem 0 0;
  font-size: 0.8rem;
  color: var(--muted-fg);
}

.fields {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.55rem 0.75rem;
  align-items: start;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  font-size: 0.72rem;
  color: var(--muted-fg);
}

.field.wide {
  grid-column: 1 / -1;
}

.field input {
  font: inherit;
  font-size: 0.9rem;
  color: var(--fg);
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.35rem 0.45rem;
}

.note {
  grid-column: 1 / -1;
  margin: 0.15rem 0 0;
  font-size: 0.78rem;
  line-height: 1.4;
  color: var(--muted-fg);
}

.check {
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.88rem;
  color: var(--fg);
  margin-top: 0.15rem;
}
</style>
