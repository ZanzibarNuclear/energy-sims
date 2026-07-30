<script setup lang="ts">
import type { OperatorInputs, PlantParams } from "../lib/plantParams";
import type { DerivedGeometry } from "../lib/compileSite";

defineProps<{
  params: PlantParams;
  operator: OperatorInputs;
  derived: DerivedGeometry | null;
  complete: boolean;
}>();

const emit = defineEmits<{
  "update:params": [PlantParams];
  "update:operator": [OperatorInputs];
}>();

function num(
  obj: "params" | "operator",
  path: string[],
  raw: string,
  params: PlantParams,
  operator: OperatorInputs,
) {
  const n = Number(raw);
  if (!Number.isFinite(n) && raw !== "") return;
  if (obj === "operator") {
    const next = structuredClone(operator);
    // @ts-expect-error dynamic
    next[path[0]!] = n;
    emit("update:operator", next);
    return;
  }
  const next = structuredClone(params);
  let cur: Record<string, unknown> = next as unknown as Record<string, unknown>;
  for (let i = 0; i < path.length - 1; i++) {
    cur = cur[path[i]!] as Record<string, unknown>;
  }
  cur[path[path.length - 1]!] = n;
  emit("update:params", next);
}

function str(path: string[], raw: string, params: PlantParams) {
  const next = structuredClone(params);
  let cur: Record<string, unknown> = next as unknown as Record<string, unknown>;
  for (let i = 0; i < path.length - 1; i++) {
    cur = cur[path[i]!] as Record<string, unknown>;
  }
  cur[path[path.length - 1]!] = raw;
  emit("update:params", next);
}

function bool(path: string[], value: boolean, params: PlantParams) {
  const next = structuredClone(params);
  let cur: Record<string, unknown> = next as unknown as Record<string, unknown>;
  for (let i = 0; i < path.length - 1; i++) {
    cur = cur[path[i]!] as Record<string, unknown>;
  }
  cur[path[path.length - 1]!] = value;
  emit("update:params", next);
}

function opBool(key: keyof OperatorInputs, value: boolean, operator: OperatorInputs) {
  emit("update:operator", { ...operator, [key]: value });
}
</script>

<template>
  <div class="plant-form">
    <h2>Plant</h2>
    <p v-if="!complete" class="hint">Geometry fields unlock after intake + turbine are placed.</p>

    <div v-if="derived" class="derived">
      <h3>Derived geometry</h3>
      <ul>
        <li>
          Head H<sub>g</sub>:
          <strong>{{ derived.grossHeadM.toFixed(2) }} m</strong>
        </li>
        <li>
          Path length L:
          <strong>{{ derived.lengthM.toFixed(2) }} m</strong>
        </li>
        <li>
          Minor K (base + bends):
          <strong>{{ derived.minorLossCoefficient.toFixed(3) }}</strong>
          <span class="dim">
            ({{ derived.baseMinorK.toFixed(2) }} + {{ derived.bendMinorK.toFixed(3) }})
          </span>
        </li>
      </ul>
    </div>

    <label class="field">
      <span>Plant id</span>
      <input
        type="text"
        :value="params.id"
        @change="str(['id'], ($event.target as HTMLInputElement).value, params)"
      />
    </label>
    <label class="field">
      <span>Label</span>
      <input
        type="text"
        :value="params.label"
        @change="str(['label'], ($event.target as HTMLInputElement).value, params)"
      />
    </label>

    <h3>Stream</h3>
    <label class="field">
      <span>Available flow (m³/s)</span>
      <input
        type="number"
        step="0.001"
        min="0"
        :value="params.stream.availableFlowM3s"
        @change="
          num(
            'params',
            ['stream', 'availableFlowM3s'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
        "
      />
    </label>

    <h3>Penstock</h3>
    <label class="field">
      <span>Diameter (m)</span>
      <input
        type="number"
        step="0.01"
        min="0.01"
        :value="params.penstock.diameterM"
        @change="
          num(
            'params',
            ['penstock', 'diameterM'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
        "
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
          num(
            'params',
            ['penstock', 'frictionFactor'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
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
          num(
            'params',
            ['penstock', 'baseMinorLossCoefficient'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
        "
      />
    </label>

    <details class="adv">
      <summary>Advanced overrides</summary>
      <label class="check">
        <input
          type="checkbox"
          :checked="params.penstock.overrideHead"
          @change="
            bool(
              ['penstock', 'overrideHead'],
              ($event.target as HTMLInputElement).checked,
              params,
            )
          "
        />
        Override gross head (m)
      </label>
      <input
        type="number"
        step="0.1"
        :disabled="!params.penstock.overrideHead"
        :value="params.penstock.overrideGrossHeadM"
        @change="
          num(
            'params',
            ['penstock', 'overrideGrossHeadM'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
        "
      />
      <label class="check">
        <input
          type="checkbox"
          :checked="params.penstock.overrideLength"
          @change="
            bool(
              ['penstock', 'overrideLength'],
              ($event.target as HTMLInputElement).checked,
              params,
            )
          "
        />
        Override length (m)
      </label>
      <input
        type="number"
        step="0.1"
        :disabled="!params.penstock.overrideLength"
        :value="params.penstock.overrideLengthM"
        @change="
          num(
            'params',
            ['penstock', 'overrideLengthM'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
        "
      />
      <label class="check">
        <input
          type="checkbox"
          :checked="params.penstock.overrideMinorLoss"
          @change="
            bool(
              ['penstock', 'overrideMinorLoss'],
              ($event.target as HTMLInputElement).checked,
              params,
            )
          "
        />
        Override total minor K
      </label>
      <input
        type="number"
        step="0.05"
        :disabled="!params.penstock.overrideMinorLoss"
        :value="params.penstock.overrideMinorLossCoefficient"
        @change="
          num(
            'params',
            ['penstock', 'overrideMinorLossCoefficient'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
        "
      />
    </details>

    <h3>Turbine / generator</h3>
    <label class="field">
      <span>Turbine η</span>
      <input
        type="number"
        step="0.01"
        min="0"
        max="1"
        :value="params.turbine.efficiency"
        @change="
          num(
            'params',
            ['turbine', 'efficiency'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
        "
      />
    </label>
    <label class="field">
      <span>Generator η</span>
      <input
        type="number"
        step="0.01"
        min="0"
        max="1"
        :value="params.generator.efficiency"
        @change="
          num(
            'params',
            ['generator', 'efficiency'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
        "
      />
    </label>
    <label class="field">
      <span>Design flow (m³/s)</span>
      <input
        type="number"
        step="0.001"
        :value="params.turbine.designFlowM3s"
        @change="
          num(
            'params',
            ['turbine', 'designFlowM3s'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
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
          num(
            'params',
            ['generator', 'ratedPowerKw'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
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
          num(
            'params',
            ['turbine', 'dynamics', 'powerRampUpS'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
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
          num(
            'params',
            ['turbine', 'dynamics', 'powerRampDownS'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
        "
      />
    </label>

    <h3>Operator</h3>
    <label class="field">
      <span>Gate opening (0–1)</span>
      <input
        type="number"
        step="0.05"
        min="0"
        max="1"
        :value="operator.gateOpening"
        @change="
          num(
            'operator',
            ['gateOpening'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
        "
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
        @change="
          num(
            'operator',
            ['debrisClogFraction'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
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
        @change="
          num(
            'operator',
            ['leakageFraction'],
            ($event.target as HTMLInputElement).value,
            params,
            operator,
          )
        "
      />
    </label>
    <label class="check">
      <input
        type="checkbox"
        :checked="operator.online"
        @change="opBool('online', ($event.target as HTMLInputElement).checked, operator)"
      />
      Online
    </label>
  </div>
</template>

<style scoped>
.plant-form h2 {
  margin: 0 0 0.5rem;
  font-size: 0.95rem;
  font-weight: 650;
}

.plant-form h3 {
  margin: 0.85rem 0 0.4rem;
  font-size: 0.78rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--muted-fg);
}

.hint {
  margin: 0 0 0.5rem;
  font-size: 0.8rem;
  color: var(--muted-fg);
}

.derived {
  margin-bottom: 0.65rem;
  padding: 0.5rem 0.6rem;
  border-radius: 8px;
  background: var(--bg);
  border: 1px solid var(--border);
  font-size: 0.82rem;
}

.derived h3 {
  margin: 0 0 0.35rem;
}

.derived ul {
  margin: 0;
  padding-left: 1.1rem;
}

.dim {
  color: var(--muted-fg);
  font-size: 0.9em;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  margin-bottom: 0.45rem;
  font-size: 0.75rem;
  color: var(--muted-fg);
}

.field input,
.adv input[type="number"] {
  font: inherit;
  font-size: 0.88rem;
  color: var(--fg);
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.3rem 0.45rem;
}

.adv {
  margin: 0.5rem 0;
  font-size: 0.8rem;
}

.adv summary {
  cursor: pointer;
  color: var(--muted-fg);
  margin-bottom: 0.35rem;
}

.adv input[type="number"] {
  width: 100%;
  margin-bottom: 0.4rem;
}

.check {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.8rem;
  margin: 0.35rem 0;
  color: var(--fg);
}
</style>
