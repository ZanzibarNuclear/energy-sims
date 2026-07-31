<script setup lang="ts">
import { computed } from "vue";
import type { TrialSample } from "../lib/trialTypes";

const props = defineProps<{
  samples: TrialSample[];
  /** Which series to plot. */
  series: "power" | "speed";
  height?: number;
}>();

const W = 480;
const H = computed(() => props.height ?? 140);
const pad = { l: 44, r: 12, t: 12, b: 28 };
const TIME_GRID_S = 10;

const points = computed(() => {
  const samples = props.samples;
  if (samples.length < 2) {
    return { line: "", area: "", maxY: 1, maxT: 1, vLines: [] as { x: number; t: number }[] };
  }

  const ys =
    props.series === "power"
      ? samples.map((s) => s.electricalPowerKw)
      : samples.map((s) => s.turbineSpeedRpm);
  const ts = samples.map((s) => s.simTimeS);
  const maxY = Math.max(...ys, 1e-9);
  const maxT = Math.max(...ts, 1e-9);
  const innerW = W - pad.l - pad.r;
  const innerH = H.value - pad.t - pad.b;

  const coords = samples.map((_, i) => {
    const x = pad.l + (ts[i]! / maxT) * innerW;
    const y = pad.t + innerH - (ys[i]! / maxY) * innerH;
    return [x, y] as const;
  });

  const line = coords.map(([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`).join(" ");
  const area =
    `${pad.l},${pad.t + innerH} ` +
    line +
    ` ${pad.l + innerW},${pad.t + innerH}`;

  // Vertical grid every 10 s of sim time (including 0).
  const vLines: { x: number; t: number }[] = [];
  for (let t = 0; t <= maxT + 1e-9; t += TIME_GRID_S) {
    vLines.push({
      t,
      x: pad.l + (t / maxT) * innerW,
    });
  }

  return { line, area, maxY, maxT, vLines };
});

const title = computed(() =>
  props.series === "power" ? "Electrical power (kW)" : "Turbine speed (rpm)",
);

const yUnit = computed(() => (props.series === "power" ? "kW" : "rpm"));

const plotTop = computed(() => pad.t);
const plotBottom = computed(() => H.value - pad.b);
</script>

<template>
  <div class="chart">
    <div class="title">{{ title }}</div>
    <svg
      v-if="samples.length >= 2"
      :viewBox="`0 0 ${W} ${H}`"
      class="svg"
      role="img"
      :aria-label="title"
    >
      <line
        v-for="g in points.vLines"
        :key="'v' + g.t"
        :x1="g.x"
        :x2="g.x"
        :y1="plotTop"
        :y2="plotBottom"
        class="vgrid"
      />
      <polyline
        v-if="points.area"
        :points="points.area"
        class="area"
      />
      <polyline :points="points.line" class="line" fill="none" />
      <text
        v-for="g in points.vLines"
        :key="'tl' + g.t"
        :x="g.x"
        :y="H - 8"
        text-anchor="middle"
        class="axis"
      >
        {{ g.t }} s
      </text>
      <text :x="4" :y="pad.t + 10" class="axis">
        {{ points.maxY.toFixed(series === "power" ? 1 : 0) }} {{ yUnit }}
      </text>
      <text :x="4" :y="H - pad.b" class="axis">0 {{ yUnit }}</text>
    </svg>
    <p v-else class="empty">Run a trial to plot the series.</p>
  </div>
</template>

<style scoped>
.chart {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg);
  padding: 0.4rem 0.5rem 0.25rem;
}

.title {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--muted-fg);
  margin-bottom: 0.15rem;
}

.svg {
  width: 100%;
  height: auto;
  display: block;
}

.line {
  stroke: var(--accent);
  stroke-width: 2;
  stroke-linejoin: round;
  stroke-linecap: round;
}

.area {
  fill: color-mix(in srgb, var(--accent) 18%, transparent);
  stroke: none;
}

.vgrid {
  stroke: var(--fg);
  stroke-opacity: 0.12;
  stroke-width: 1;
}

.axis {
  font-size: 10px;
  fill: var(--muted-fg);
}

.empty {
  margin: 0;
  padding: 1.5rem 0.5rem;
  text-align: center;
  font-size: 0.8rem;
  color: var(--muted-fg);
}
</style>
