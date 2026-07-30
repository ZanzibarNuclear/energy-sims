<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import {
  createEnergySimClient,
  defaultEngineUrl,
  type HealthResponse,
} from "../lib/energySimClient";

const props = withDefaults(
  defineProps<{
    /** Poll interval in ms; 0 disables after first check. */
    pollMs?: number;
  }>(),
  { pollMs: 5000 },
);

type StatusKind = "checking" | "ok" | "down";

const baseUrl = ref(defaultEngineUrl());
const status = ref<StatusKind>("checking");
const detail = ref("");
const lastChecked = ref<Date | null>(null);

let timer: ReturnType<typeof setInterval> | null = null;

async function check() {
  const client = createEnergySimClient({ baseUrl: baseUrl.value });
  try {
    const health = (await client.health()) as HealthResponse;
    status.value = health?.ok === false ? "down" : "ok";
    const engine = health?.engine ? String(health.engine) : "energy-sim";
    detail.value = engine;
  } catch (e) {
    status.value = "down";
    detail.value = e instanceof Error ? e.message : "unreachable";
  } finally {
    lastChecked.value = new Date();
  }
}

function startPolling() {
  if (timer) clearInterval(timer);
  timer = null;
  void check();
  if (props.pollMs > 0) {
    timer = setInterval(() => {
      void check();
    }, props.pollMs);
  }
}

onMounted(startPolling);
onUnmounted(() => {
  if (timer) clearInterval(timer);
});

watch(
  () => props.pollMs,
  () => startPolling(),
);

defineExpose({ check, baseUrl, status });
</script>

<template>
  <div class="server-status" :data-status="status" title="energy-sim-server">
    <span class="dot" aria-hidden="true" />
    <div class="text">
      <div class="label">
        <template v-if="status === 'checking'">Checking engine…</template>
        <template v-else-if="status === 'ok'">Engine connected</template>
        <template v-else>Engine offline</template>
      </div>
      <div class="meta">
        <code>{{ baseUrl }}</code>
        <span v-if="status === 'ok' && detail" class="sep">·</span>
        <span v-if="status === 'ok' && detail">{{ detail }}</span>
        <span v-if="status === 'down' && detail" class="err">{{ detail }}</span>
      </div>
      <p v-if="status === 'down'" class="hint">
        Start the server:
        <code>cargo run -p energy-sim-server -- --listen 127.0.0.1:8787</code>
      </p>
    </div>
    <button type="button" class="retry" :disabled="status === 'checking'" @click="check">
      Retry
    </button>
  </div>
</template>

<style scoped>
.server-status {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  padding: 0.65rem 0.85rem;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--panel);
}

.dot {
  width: 0.65rem;
  height: 0.65rem;
  margin-top: 0.35rem;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--muted);
}

.server-status[data-status="checking"] .dot {
  background: #c9a227;
  box-shadow: 0 0 0 3px rgba(201, 162, 39, 0.25);
}

.server-status[data-status="ok"] .dot {
  background: #3d9a5f;
  box-shadow: 0 0 0 3px rgba(61, 154, 95, 0.25);
}

.server-status[data-status="down"] .dot {
  background: #c44;
  box-shadow: 0 0 0 3px rgba(204, 68, 68, 0.2);
}

.text {
  flex: 1;
  min-width: 0;
}

.label {
  font-weight: 600;
  font-size: 0.95rem;
}

.meta {
  margin-top: 0.15rem;
  font-size: 0.8rem;
  color: var(--muted-fg);
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  align-items: center;
}

.meta code,
.hint code {
  font-size: 0.78rem;
  background: var(--code-bg);
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
}

.sep {
  opacity: 0.5;
}

.err {
  color: #c44;
}

.hint {
  margin: 0.4rem 0 0;
  font-size: 0.78rem;
  color: var(--muted-fg);
  line-height: 1.4;
}

.retry {
  flex-shrink: 0;
  font: inherit;
  font-size: 0.8rem;
  padding: 0.35rem 0.65rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--fg);
  cursor: pointer;
}

.retry:hover:not(:disabled) {
  border-color: var(--accent);
}

.retry:disabled {
  opacity: 0.5;
  cursor: default;
}
</style>
