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

type SectionId = "config" | "intake" | "penstock" | "turbine" | "generator";

const sections: {
  id: SectionId;
  label: string;
  blurb: string;
}[] = [
  { id: "config", label: "Config", blurb: "Name metadata for this site" },
  { id: "intake", label: "Intake", blurb: "Flow into the penstock" },
  { id: "penstock", label: "Penstock", blurb: "Pipe size, friction, leakage" },
  { id: "turbine", label: "Turbine", blurb: "η, flow limits, gate, ramps" },
  { id: "generator", label: "Generator", blurb: "η and nameplate cap" },
];

const active = ref<SectionId>("intake");

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
  // Online is session lifecycle (Run tab), not a hardware setting.
  emit("update:operator", { ...props.operator, [key]: n, online: true });
}

function summary(id: SectionId): string {
  const p = props.params;
  const o = props.operator;
  switch (id) {
    case "config":
      return p.id || "—";
    case "intake":
      return `${p.stream.availableFlowM3s} m³/s · debris ${o.debrisClogFraction}`;
    case "penstock":
      return `Ø ${p.penstock.diameterM} m · leak ${o.leakageFraction}`;
    case "turbine":
      return `η ${p.turbine.efficiency} · gate ${o.gateOpening}`;
    case "generator":
      return `η ${p.generator.efficiency}`;
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

      <!-- Config metadata -->
      <div v-if="active === 'config'" class="fields">
        <label class="field wide">
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
        <p class="note">
          Identifies this configuration in saves and engine plant JSON. Not a physical component.
        </p>
      </div>

      <!-- Intake -->
      <div v-else-if="active === 'intake'" class="fields">
        <label class="field">
          <span>Intake flow available (m³/s)</span>
          <input
            type="number"
            step="0.001"
            min="0"
            :value="params.stream.availableFlowM3s"
            @change="setNum(['stream', 'availableFlowM3s'], ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="field">
          <span>Debris / screen clog (0–1)</span>
          <input
            type="number"
            step="0.05"
            min="0"
            max="1"
            :value="operator.debrisClogFraction"
            @change="setOpNum('debrisClogFraction', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <p class="note">
          Flow into the intake (not the whole creek). Debris reduces capture and adds intake head
          loss at the trash rack / screen.
        </p>
      </div>

      <!-- Penstock -->
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
          <span>Base minor K (entrance)</span>
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
        <p class="note">
          Head and pipe length come from Layout. Each bend adds minor-loss K from its turn angle
          (no per-bend editor). Leakage drops flow before the turbine, not head.
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
          <span>Gate / admission (0–1)</span>
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
          <span>Design flow (m³/s)</span>
          <input
            type="number"
            step="0.001"
            :value="params.turbine.designFlowM3s"
            @change="setNum(['turbine', 'designFlowM3s'], ($event.target as HTMLInputElement).value)"
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
        <p class="note">
          Gate is the admission valve (how much of the available intake flow is admitted to the
          turbine). Online/offline is controlled on the Run tab when you play or stop a session.
        </p>
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
        <p class="note">
          Electrical efficiency only. The lab does not apply a generator nameplate cap so you can
          see full power from the layout and losses.
        </p>
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
</style>
