<script setup lang="ts">
import { computed, ref } from "vue";
import {
  createEnergySimClient,
  defaultEngineUrl,
  type EnergySimClient,
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

/** Sim-time durations. Longer runs are mostly flat after the ramp, but give wall time to hit Stop. */
const DURATION_OPTIONS = [
  { secs: 30, label: "30 s" },
  { secs: 45, label: "45 s" },
  { secs: 60, label: "60 s" },
  { secs: 600, label: "10 min" },
] as const;
const STEP_SECS = 1;

const durationSecs = ref(30);
const spinUpDemo = ref(true);
/** When true, wait ~1 real second per simulated second. Default: as fast as the engine allows. */
const wallClockMode = ref(false);
const running = ref(false);
const error = ref("");
const trial = ref<TrialResult | null>(null);
const liveSessionId = ref<string | null>(null);
const liveGate = ref(1);
/** Set by Stop while Play is stepping through sim time. */
const stopRequested = ref(false);

const snapshot = computed(() => trial.value?.snapshot ?? null);

/** Label matches spin-up demo mode, not run/idle state. */
const stopButtonLabel = computed(() =>
  spinUpDemo.value ? "⏹ Spin down" : "⏹ Stop",
);

const statusNote = computed(() => {
  if (running.value) {
    return wallClockMode.value
      ? "Wall-clock mode: about 1 real second per simulated second. Use Stop / Spin down anytime."
      : "Fast mode: sim seconds as quickly as the engine can compute. Charts update each sim second.";
  }
  if (wallClockMode.value) {
    return "Wall-clock mode is on — a 30 s run takes about half a minute of real time (good for watching ramps).";
  }
  return "Fast mode (default): sim time flies. Switch on wall clock to watch ramps in real time. Spin-up checkbox labels the stop control.";
});

function hydroInputCmd(op: Partial<OperatorInputs> & Record<string, unknown>) {
  return {
    type: "set_hydro_input",
    gate_opening: op.gateOpening,
    debris_clog_fraction: op.debrisClogFraction,
    leakage_fraction: op.leakageFraction,
    online: op.online ?? true,
  };
}

function energyFrom(report: {
  energyIntervalKwh?: number;
  energy_interval_kwh?: number;
}): number {
  return Number(report.energyIntervalKwh ?? report.energy_interval_kwh ?? 0);
}

async function refreshTrial(
  client: EnergySimClient,
  sessionId: string,
  snapshot: Snapshot,
  energyIntervalKwh: number,
  mode: "interval" | "spinup",
) {
  const hist = await client.history(sessionId);
  const samples = samplesFromHistory(hist);
  const lastT = samples.length ? samples[samples.length - 1]!.simTimeS : snapshot.simTimeS ?? 0;
  trial.value = {
    sessionId,
    samples,
    snapshot,
    energyIntervalKwh,
    durationSecs: lastT,
    mode,
  };
}

/**
 * Advance sim time in small steps so the UI can update and Stop can interrupt.
 */
async function advanceInSteps(
  client: EnergySimClient,
  sessionId: string,
  totalSecs: number,
  mode: "interval" | "spinup",
  energySoFar: number,
): Promise<{ energy: number; stopped: boolean; snapshot: Snapshot | null }> {
  let left = totalSecs;
  let energy = energySoFar;
  let lastSnap: Snapshot | null = trial.value?.snapshot ?? null;

  while (left > 1e-9 && !stopRequested.value) {
    const dt = Math.min(STEP_SECS, left);
    const report = (await client.advance(sessionId, { durationSecs: dt })) as {
      snapshot: Snapshot;
      energyIntervalKwh?: number;
      energy_interval_kwh?: number;
    };
    energy += energyFrom(report);
    lastSnap = report.snapshot;
    left -= dt;
    await refreshTrial(client, sessionId, report.snapshot, energy, mode);
    // Yield for paint / Stop; wall-clock mode paces ~1 real second per sim second.
    const waitMs = wallClockMode.value ? Math.round(dt * 1000) : 0;
    await new Promise((r) => setTimeout(r, waitMs));
  }

  return { energy, stopped: stopRequested.value, snapshot: lastSnap };
}

async function runTrial() {
  if (!props.enabled || !props.plant) {
    error.value = "Complete the site so a plant can be compiled.";
    return;
  }
  running.value = true;
  stopRequested.value = false;
  error.value = "";
  trial.value = null;
  liveSessionId.value = null;

  try {
    const client = createEnergySimClient({ baseUrl: defaultEngineUrl() });
    const { sessionId } = await client.createSession(props.plant);
    liveSessionId.value = sessionId;
    const mode = spinUpDemo.value ? "spinup" : "interval";
    let energy = 0;

    if (spinUpDemo.value) {
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
    } else {
      await client.applyCommands(sessionId, [
        hydroInputCmd({ ...props.operator, online: true }),
      ]);
      await client.start(sessionId);
      liveGate.value = props.operator.gateOpening;
    }

    const { energy: e2, stopped } = await advanceInSteps(
      client,
      sessionId,
      durationSecs.value,
      mode,
      energy,
    );
    energy = e2;

    if (stopped) {
      await spinDown(client, sessionId, energy, mode);
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
    liveSessionId.value = null;
  } finally {
    running.value = false;
    stopRequested.value = false;
  }
}

/**
 * Spin-down: close the gate while the session stays Running, then advance
 * through the power ramp. Calling stop() first sets phase=Stopped and the
 * engine refuses advance until start() — see energy-sim-runtime session tests.
 */
async function spinDown(
  client: EnergySimClient,
  sessionId: string,
  energySoFar: number,
  mode: "interval" | "spinup",
) {
  // Ensure Running (needed if a prior stop() left phase=Stopped).
  await client.start(sessionId);
  await client.applyCommands(sessionId, [
    hydroInputCmd({
      gateOpening: 0,
      debrisClogFraction: props.operator.debrisClogFraction,
      leakageFraction: props.operator.leakageFraction,
      online: true,
    }),
  ]);
  liveGate.value = 0;
  const down = Math.max(25, props.plant?.turbine.dynamics.powerRampDownS ?? 25);
  stopRequested.value = false;
  await advanceInSteps(client, sessionId, down, mode, energySoFar);
  // Mark session stopped only after the ramp has been integrated.
  await client.stop(sessionId);
}

function requestStop() {
  if (running.value) {
    stopRequested.value = true;
    return;
  }
  // After a finished run, Stop still means “spin down from steady”.
  void stopAfterRun();
}

async function stopAfterRun() {
  if (!liveSessionId.value || running.value) return;
  running.value = true;
  error.value = "";
  try {
    const client = createEnergySimClient({ baseUrl: defaultEngineUrl() });
    await spinDown(
      client,
      liveSessionId.value,
      trial.value?.energyIntervalKwh ?? 0,
      trial.value?.mode ?? "interval",
    );
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    running.value = false;
    stopRequested.value = false;
  }
}

async function applyGateAndContinue() {
  if (!liveSessionId.value || running.value) return;
  running.value = true;
  stopRequested.value = false;
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
    await advanceInSteps(
      client,
      liveSessionId.value,
      30,
      trial.value?.mode ?? "interval",
      trial.value?.energyIntervalKwh ?? 0,
    );
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    running.value = false;
    stopRequested.value = false;
  }
}
</script>

<template>
  <section class="trial" aria-label="Run simulation">
    <div class="trial-bar">
      <h2>Simulate</h2>
      <label class="field-inline">
        Duration
        <select v-model.number="durationSecs" :disabled="running">
          <option v-for="d in DURATION_OPTIONS" :key="d.secs" :value="d.secs">
            {{ d.label }}
          </option>
        </select>
      </label>
      <label class="check">
        <input v-model="spinUpDemo" type="checkbox" :disabled="running" />
        Spin-up (gate 0 → open)
      </label>
      <label class="check" title="Wait about one real second for each simulated second">
        <input v-model="wallClockMode" type="checkbox" :disabled="running" />
        Wall clock
      </label>
      <button
        type="button"
        class="btn primary"
        :disabled="!enabled || running"
        @click="runTrial"
      >
        {{ running ? "Running…" : "▶ Play" }}
      </button>
      <button
        type="button"
        class="btn"
        :disabled="!liveSessionId || (!running && !trial)"
        @click="requestStop"
      >
        {{ stopButtonLabel }}
      </button>
    </div>

    <p class="status-note">{{ statusNote }}</p>

    <p v-if="!enabled" class="placeholder">
      Finish Layout and Equipment, then play a run against
      <code>energy-sim-server</code>.
    </p>
    <p v-else-if="error" class="err">{{ error }}</p>

    <div v-if="snapshot" class="strip" aria-label="Snapshot">
      <div>
        <span class="k">sim t</span>
        <span class="v">{{ Number(snapshot.simTimeS).toFixed(1) }} s</span>
      </div>
      <div>
        <span class="k">P_e actual</span>
        <span class="v">{{ Number(snapshot.electricalPowerKw).toFixed(1) }} kW</span>
      </div>
      <div>
        <span class="k">P_e target</span>
        <span class="v">{{ Number(snapshot.targetElectricalPowerKw).toFixed(1) }} kW</span>
      </div>
      <div>
        <span class="k">Speed</span>
        <span class="v">{{ Number(snapshot.turbineSpeedRpm).toFixed(0) }} rpm</span>
      </div>
      <div>
        <span class="k">H_net</span>
        <span class="v">{{ Number(snapshot.netHeadM).toFixed(1) }} m</span>
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

    <div v-if="liveSessionId && !running" class="mid">
      <label class="field-inline">
        Gate
        <input
          v-model.number="liveGate"
          type="number"
          min="0"
          max="1"
          step="0.05"
        />
      </label>
      <button type="button" class="btn" @click="applyGateAndContinue">
        Set gate + 30 s
      </button>
    </div>

    <div class="charts">
      <SeriesChart :samples="trial?.samples ?? []" series="power" />
      <SeriesChart :samples="trial?.samples ?? []" series="speed" />
    </div>
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

.status-note {
  margin: 0;
  font-size: 0.8rem;
  line-height: 1.4;
  color: var(--muted-fg);
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
</style>
