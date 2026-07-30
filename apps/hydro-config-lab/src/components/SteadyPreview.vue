<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  createEnergySimClient,
  defaultEngineUrl,
  type Snapshot,
} from "../lib/energySimClient";
import type { HydroPlantJson, OperatorInputs } from "../lib/plantParams";

const props = defineProps<{
  plant: HydroPlantJson | null;
  operator: OperatorInputs;
  enabled: boolean;
}>();

const loading = ref(false);
const error = ref("");
const snap = ref<Snapshot | null>(null);
let timer: ReturnType<typeof setTimeout> | null = null;

const targetKw = computed(() => snap.value?.targetElectricalPowerKw ?? null);

async function refresh() {
  if (!props.enabled || !props.plant) {
    snap.value = null;
    error.value = "";
    return;
  }
  loading.value = true;
  error.value = "";
  try {
    const client = createEnergySimClient({ baseUrl: defaultEngineUrl() });
    const { sessionId, snapshot } = await client.createSession(props.plant);
    // Apply operator inputs so gate/debris/leakage affect targets.
    // Command fields are snake_case (runtime Command enum).
    const after = await client.applyCommands(sessionId, [
      {
        type: "set_hydro_input",
        gate_opening: props.operator.gateOpening,
        debris_clog_fraction: props.operator.debrisClogFraction,
        leakage_fraction: props.operator.leakageFraction,
        online: props.operator.online,
      },
    ]);
    snap.value = (after as Snapshot) ?? (snapshot as Snapshot);
  } catch (e) {
    snap.value = null;
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

function schedule() {
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => {
    void refresh();
  }, 280);
}

watch(
  () => [props.plant, props.operator, props.enabled] as const,
  () => schedule(),
  { deep: true, immediate: true },
);
</script>

<template>
  <div class="preview">
    <h3>Steady preview</h3>
    <p v-if="!enabled" class="placeholder">Complete the site (intake + turbine) to evaluate.</p>
    <p v-else-if="loading && !snap" class="placeholder">Evaluating…</p>
    <p v-else-if="error" class="err">{{ error }}</p>
    <template v-else-if="snap">
      <dl>
        <div>
          <dt>Target P<sub>e</sub></dt>
          <dd>{{ Number(targetKw).toFixed(3) }} kW</dd>
        </div>
        <div>
          <dt>Hydraulic P</dt>
          <dd>{{ Number(snap.hydraulicPowerKw).toFixed(3) }} kW</dd>
        </div>
        <div>
          <dt>H<sub>gross</sub> / H<sub>net</sub></dt>
          <dd>
            {{ Number(snap.grossHeadM).toFixed(2) }} /
            {{ Number(snap.netHeadM).toFixed(2) }} m
          </dd>
        </div>
        <div>
          <dt>Head loss</dt>
          <dd>{{ Number(snap.headLossM).toFixed(3) }} m</dd>
        </div>
        <div>
          <dt>Flow</dt>
          <dd>{{ Number(snap.flowM3s).toFixed(4) }} m³/s</dd>
        </div>
      </dl>
      <ul v-if="snap.warnings?.length" class="warn">
        <li v-for="(w, i) in snap.warnings" :key="i">{{ w }}</li>
      </ul>
      <p class="note">
        Steady targets from engine (create session). Actual ramped power appears in trials.
        <button type="button" class="link" :disabled="loading" @click="refresh">Refresh</button>
      </p>
    </template>
  </div>
</template>

<style scoped>
.preview h3 {
  margin: 0 0 0.4rem;
  font-size: 0.82rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--muted-fg);
}

.placeholder {
  margin: 0;
  font-size: 0.85rem;
  color: var(--muted-fg);
}

.err {
  margin: 0;
  font-size: 0.85rem;
  color: #c44;
}

dl {
  margin: 0;
  display: grid;
  gap: 0.35rem;
}

dl div {
  display: flex;
  justify-content: space-between;
  gap: 0.5rem;
  font-size: 0.85rem;
}

dt {
  color: var(--muted-fg);
}

dd {
  margin: 0;
  font-variant-numeric: tabular-nums;
  font-weight: 600;
}

.warn {
  margin: 0.5rem 0 0;
  padding-left: 1.1rem;
  font-size: 0.75rem;
  color: #b8860b;
}

.note {
  margin: 0.55rem 0 0;
  font-size: 0.72rem;
  color: var(--muted-fg);
  line-height: 1.4;
}

.link {
  font: inherit;
  font-size: inherit;
  color: var(--accent);
  background: none;
  border: none;
  cursor: pointer;
  text-decoration: underline;
  padding: 0;
  margin-left: 0.25rem;
}

.link:disabled {
  opacity: 0.5;
  cursor: default;
}
</style>
