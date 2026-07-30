<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { DerivedGeometry } from "../lib/compileSite";
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
  derived: DerivedGeometry | null;
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
  <div class="side">
    <section v-if="derived" class="card">
      <h3>From layout</h3>
      <dl>
        <div>
          <dt>Head H</dt>
          <dd>{{ derived.grossHeadM.toFixed(1) }} m</dd>
        </div>
        <div>
          <dt>Pipe L</dt>
          <dd>{{ derived.lengthM.toFixed(1) }} m</dd>
        </div>
        <div>
          <dt>Minor K</dt>
          <dd>
            {{ derived.minorLossCoefficient.toFixed(2) }}
            <span class="dim">
              ({{ derived.baseMinorK.toFixed(2) }}+{{ derived.bendMinorK.toFixed(2) }})
            </span>
          </dd>
        </div>
      </dl>
    </section>
    <p v-else class="placeholder card">Complete the layout for head and length.</p>

    <section class="card">
      <h3>Steady preview</h3>
      <p v-if="!enabled" class="placeholder">Need a complete layout to evaluate.</p>
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
            <dt>H<sub>net</sub></dt>
            <dd>{{ Number(snap.netHeadM).toFixed(2) }} m</dd>
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
        <button type="button" class="link" :disabled="loading" @click="refresh">Refresh</button>
      </template>
    </section>
  </div>
</template>

<style scoped>
.side {
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
}

.card {
  padding: 0.75rem 0.85rem;
  border-radius: 8px;
  background: var(--bg);
  border: 1px solid var(--border);
}

h3 {
  margin: 0 0 0.45rem;
  font-size: 0.72rem;
  font-weight: 650;
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
  gap: 0.3rem;
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
  text-align: right;
}

.dim {
  font-weight: 400;
  color: var(--muted-fg);
  font-size: 0.85em;
}

.warn {
  margin: 0.45rem 0 0;
  padding-left: 1.1rem;
  font-size: 0.75rem;
  color: #b8860b;
}

.link {
  margin-top: 0.45rem;
  font: inherit;
  font-size: 0.78rem;
  color: var(--accent);
  background: none;
  border: none;
  cursor: pointer;
  text-decoration: underline;
  padding: 0;
}

.link:disabled {
  opacity: 0.5;
  cursor: default;
}
</style>
