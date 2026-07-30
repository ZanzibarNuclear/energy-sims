<script setup lang="ts">
import { computed, ref } from "vue";
import {
  createEnergySimClient,
  defaultEngineUrl,
  type Snapshot,
} from "../lib/energySimClient";
import type { HydroPlantJson, OperatorInputs } from "../lib/plantParams";
import { samplesFromHistory, type TrialResult } from "../lib/trialTypes";
import SeriesChart from "./SeriesChart.vue";

const props = defineProps<{
  plant: HydroPlantJson | null;
  operator: OperatorInputs;
  enabled: boolean;
}>();

const durationSecs = ref(60);
const spinUpDemo = ref(true);
const running = ref(false);
const error = ref("");
const trial = ref<TrialResult | null>(null);
const liveSessionId = ref<string | null>(null);
const liveGate = ref(1);

const snapshot = computed(() => trial.value?.snapshot ?? null);

function hydroInputCmd(op: Partial<OperatorInputs> & Record<string, unknown>) {
  return {
    type: "set_hydro_input",
    gate_opening: op.gateOpening,
    debris_clog_fraction: op.debrisClogFraction,
    leakage_fraction: op.leakageFraction,
    online: op.online,
  };
}

async function runTrial() {
  if (!props.enabled || !props.plant) {
    error.value = "Complete the site so a plant can be compiled.";
    return;
  }
  running.value = true;
  error.value = "";
  trial.value = null;
  liveSessionId.value = null;
  try {
    const client = createEnergySimClient({ baseUrl: defaultEngineUrl() });
    const { sessionId } = await client.createSession(props.plant);
    liveSessionId.value = sessionId;

    if (spinUpDemo.value) {
      // Start offline/closed, then open gate to show ramp curve.
      await client.applyCommands(sessionId, [
        hydroInputCmd({
          ...props.operator,
          gateOpening: 0,
          online: true,
        }),
      ]);
      await client.start(sessionId);
      await client.advance(sessionId, { durationSecs: 1 });
      await client.applyCommands(sessionId, [
        hydroInputCmd({
          ...props.operator,
          gateOpening: props.operator.gateOpening > 0 ? props.operator.gateOpening : 1,
          online: true,
        }),
      ]);
      liveGate.value = props.operator.gateOpening > 0 ? props.operator.gateOpening : 1;
      const report = (await client.advance(sessionId, {
        durationSecs: durationSecs.value,
      })) as {
        snapshot: Snapshot;
        energyIntervalKwh?: number;
        energy_interval_kwh?: number;
      };
      const hist = await client.history(sessionId);
      trial.value = {
        sessionId,
        samples: samplesFromHistory(hist),
        snapshot: report.snapshot,
        energyIntervalKwh: Number(
          report.energyIntervalKwh ?? report.energy_interval_kwh ?? 0,
        ),
        durationSecs: durationSecs.value + 1,
        mode: "spinup",
      };
    } else {
      await client.applyCommands(sessionId, [hydroInputCmd(props.operator)]);
      await client.start(sessionId);
      liveGate.value = props.operator.gateOpening;
      const report = (await client.advance(sessionId, {
        durationSecs: durationSecs.value,
      })) as {
        snapshot: Snapshot;
        energyIntervalKwh?: number;
        energy_interval_kwh?: number;
      };
      const hist = await client.history(sessionId);
      trial.value = {
        sessionId,
        samples: samplesFromHistory(hist),
        snapshot: report.snapshot,
        energyIntervalKwh: Number(
          report.energyIntervalKwh ?? report.energy_interval_kwh ?? 0,
        ),
        durationSecs: durationSecs.value,
        mode: "interval",
      };
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
    liveSessionId.value = null;
  } finally {
    running.value = false;
  }
}

async function applyGateAndContinue() {
  if (!liveSessionId.value) return;
  running.value = true;
  error.value = "";
  try {
    const client = createEnergySimClient({ baseUrl: defaultEngineUrl() });
    await client.applyCommands(liveSessionId.value, [
      hydroInputCmd({
        ...props.operator,
        gateOpening: liveGate.value,
        online: true,
      }),
    ]);
    const report = (await client.advance(liveSessionId.value, {
      durationSecs: 30,
    })) as { snapshot: Snapshot; energyIntervalKwh?: number };
    const hist = await client.history(liveSessionId.value);
    trial.value = {
      sessionId: liveSessionId.value,
      samples: samplesFromHistory(hist),
      snapshot: report.snapshot,
      energyIntervalKwh: Number(report.energyIntervalKwh ?? 0),
      durationSecs: (trial.value?.durationSecs ?? 0) + 30,
      mode: trial.value?.mode ?? "interval",
    };
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    running.value = false;
  }
}

async function stopTrial() {
  if (!liveSessionId.value) return;
  running.value = true;
  error.value = "";
  try {
    const client = createEnergySimClient({ baseUrl: defaultEngineUrl() });
    await client.stop(liveSessionId.value);
    // Advance to show spin-down curve.
    const report = (await client.advance(liveSessionId.value, {
      durationSecs: Math.max(25, props.plant?.turbine.dynamics.powerRampDownS ?? 25),
    })) as { snapshot: Snapshot; energyIntervalKwh?: number };
    const hist = await client.history(liveSessionId.value);
    trial.value = {
      sessionId: liveSessionId.value,
      samples: samplesFromHistory(hist),
      snapshot: report.snapshot,
      energyIntervalKwh: Number(report.energyIntervalKwh ?? 0),
      durationSecs: (trial.value?.durationSecs ?? 0) + 25,
      mode: trial.value?.mode ?? "interval",
    };
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    running.value = false;
  }
}
</script>

<template>
  <section class="trial" aria-label="Trial runner">
    <div class="trial-bar">
      <h2>Trial</h2>
      <label class="field-inline">
        Duration
        <select v-model.number="durationSecs" :disabled="running">
          <option :value="30">30 s</option>
          <option :value="60">60 s</option>
          <option :value="120">120 s</option>
        </select>
      </label>
      <label class="check">
        <input v-model="spinUpDemo" type="checkbox" :disabled="running" />
        Spin-up demo (gate 0 → open)
      </label>
      <button
        type="button"
        class="btn primary"
        :disabled="!enabled || running"
        @click="runTrial"
      >
        {{ running ? "Running…" : "Run" }}
      </button>
      <button
        type="button"
        class="btn"
        :disabled="!liveSessionId || running"
        @click="stopTrial"
      >
        Stop (spin-down)
      </button>
    </div>

    <p v-if="!enabled" class="placeholder">
      Place intake and turbine, then run a trial against <code>energy-sim-server</code>.
    </p>
    <p v-else-if="error" class="err">{{ error }}</p>

    <div v-if="snapshot" class="strip" aria-label="Snapshot">
      <div>
        <span class="k">sim t</span>
        <span class="v">{{ Number(snapshot.simTimeS).toFixed(1) }} s</span>
      </div>
      <div>
        <span class="k">P_e actual</span>
        <span class="v">{{ Number(snapshot.electricalPowerKw).toFixed(3) }} kW</span>
      </div>
      <div>
        <span class="k">P_e target</span>
        <span class="v">{{ Number(snapshot.targetElectricalPowerKw).toFixed(3) }} kW</span>
      </div>
      <div>
        <span class="k">Speed</span>
        <span class="v">{{ Number(snapshot.turbineSpeedRpm).toFixed(0) }} rpm</span>
      </div>
      <div>
        <span class="k">H_net</span>
        <span class="v">{{ Number(snapshot.netHeadM).toFixed(2) }} m</span>
      </div>
      <div>
        <span class="k">Energy Δ</span>
        <span class="v">{{ (trial?.energyIntervalKwh ?? 0).toFixed(4) }} kWh</span>
      </div>
      <div>
        <span class="k">Phase</span>
        <span class="v">{{ snapshot.phase }}</span>
      </div>
    </div>

    <div v-if="liveSessionId" class="mid">
      <label class="field-inline">
        Gate
        <input
          v-model.number="liveGate"
          type="number"
          min="0"
          max="1"
          step="0.05"
          :disabled="running"
        />
      </label>
      <button type="button" class="btn" :disabled="running" @click="applyGateAndContinue">
        Set gate + 30 s
      </button>
      <span class="hint">e.g. set 0 for drought / closed, or 1 for full open</span>
    </div>

    <div class="charts">
      <SeriesChart :samples="trial?.samples ?? []" series="power" />
      <SeriesChart :samples="trial?.samples ?? []" series="speed" />
    </div>

    <ul v-if="snapshot?.warnings?.length" class="warn">
      <li v-for="(w, i) in snapshot.warnings" :key="i">{{ w }}</li>
    </ul>
  </section>
</template>

<style scoped>
.trial {
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--panel);
  padding: 0.85rem 1rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.trial-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

.trial-bar h2 {
  margin: 0;
  margin-right: auto;
  font-size: 0.95rem;
  font-weight: 650;
}

.field-inline {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.8rem;
  color: var(--muted-fg);
}

.field-inline select,
.field-inline input {
  font: inherit;
  font-size: 0.85rem;
  color: var(--fg);
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.25rem 0.4rem;
}

.check {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.8rem;
  color: var(--fg);
}

.btn {
  font: inherit;
  font-size: 0.85rem;
  padding: 0.4rem 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--fg);
  cursor: pointer;
}

.btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.btn.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

.placeholder {
  margin: 0;
  font-size: 0.85rem;
  color: var(--muted-fg);
}

.placeholder code {
  font-size: 0.8em;
  background: var(--code-bg);
  padding: 0.05rem 0.3rem;
  border-radius: 3px;
}

.err {
  margin: 0;
  font-size: 0.85rem;
  color: #c44;
}

.strip {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem 1.1rem;
  padding: 0.55rem 0.7rem;
  border-radius: 8px;
  background: var(--bg);
  border: 1px solid var(--border);
}

.strip .k {
  display: block;
  font-size: 0.68rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--muted-fg);
}

.strip .v {
  font-size: 0.9rem;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
}

.mid {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem;
}

.hint {
  font-size: 0.75rem;
  color: var(--muted-fg);
}

.charts {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.65rem;
}

@media (max-width: 720px) {
  .charts {
    grid-template-columns: 1fr;
  }
}

.warn {
  margin: 0;
  padding-left: 1.1rem;
  font-size: 0.78rem;
  color: #b8860b;
}
</style>
