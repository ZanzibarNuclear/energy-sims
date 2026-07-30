<script setup lang="ts">
import { computed, ref } from "vue";
import {
  addBend,
  isSiteComplete,
  placeIntake,
  placeTurbine,
  profilePoints,
  siteStatusMessage,
  type Site,
  type SiteSelection,
  type ToolId,
} from "../lib/site";
import {
  clampPoint,
  DEFAULT_VIEW,
  displayToWorld,
  elevationOriginZ,
  GRID_STEP_M,
  markerRadiusM,
  snapPoint,
  ticks,
  toSvgViewBox,
  worldToDisplay,
  type WorldBounds,
} from "../lib/viewBox";

const props = defineProps<{
  site: Site;
  selection: SiteSelection;
  tool: ToolId;
}>();

const emit = defineEmits<{
  "update:site": [Site];
  "update:selection": [SiteSelection];
  "update:tool": [ToolId];
}>();

const svgRef = ref<SVGSVGElement | null>(null);
const dragging = ref<SiteSelection>(null);
/** Freeze elev origin while dragging so moving the turbine does not runaway. */
const dragElevOrigin = ref<number | null>(null);
const snapToGrid = ref(true);

/** Fixed display window (does not auto-zoom while dragging). */
const view = ref<WorldBounds>({ ...DEFAULT_VIEW });

/**
 * Elevation 0 on the plot = turbine elevation when a turbine exists.
 * Horizontal axis is plan distance s (not pipe length).
 */
const elevOriginLive = computed(() => elevationOriginZ(props.site.turbine?.zM));
const elevOrigin = computed(() =>
  dragElevOrigin.value != null ? dragElevOrigin.value : elevOriginLive.value,
);

const viewBox = computed(() => toSvgViewBox(view.value));
const markR = computed(() => markerRadiusM(view.value));
const fontM = computed(() => Math.max(3.2, (view.value.sMax - view.value.sMin) * 0.018));

const sGrid = computed(() => ticks(view.value.sMin, view.value.sMax, GRID_STEP_M));
const zGrid = computed(() => ticks(view.value.zMin, view.value.zMax, GRID_STEP_M));

function toDisplay(p: { sM: number; zM: number }) {
  return worldToDisplay(p.sM, p.zM, elevOrigin.value);
}

function fromDisplay(s: number, z: number) {
  let d = clampPoint(s, z, view.value);
  if (snapToGrid.value) {
    d = snapPoint(d.sM, d.zM, GRID_STEP_M);
    d = clampPoint(d.sM, d.zM, view.value);
  }
  return displayToWorld(d.sM, d.zM, elevOrigin.value);
}

const triangle = computed(() => {
  if (!props.site.intake || !props.site.turbine) return null;
  const a = props.site.intake;
  const t = props.site.turbine;
  const aD = toDisplay(a);
  const tD = toDisplay(t);
  const runM = Math.abs(t.sM - a.sM);
  const headM = a.zM - t.zM;
  const pts = profilePoints(props.site);
  let pipeM = 0;
  for (let i = 1; i < pts.length; i++) {
    const p0 = pts[i - 1]!;
    const p1 = pts[i]!;
    pipeM += Math.hypot(p1.sM - p0.sM, p1.zM - p0.zM);
  }
  // Right angle at (intake.s, turbine elev) in display space
  const corner = { s: aD.s, z: tD.z };
  return {
    intakeD: aD,
    turbineD: tD,
    corner,
    runM,
    headM,
    pipeM,
  };
});

function clientToDisplay(ev: PointerEvent | MouseEvent): { s: number; z: number } | null {
  const svg = svgRef.value;
  if (!svg) return null;
  const pt = svg.createSVGPoint();
  pt.x = ev.clientX;
  pt.y = ev.clientY;
  const ctm = svg.getScreenCTM();
  if (!ctm) return null;
  const local = pt.matrixTransform(ctm.inverse());
  return { s: local.x, z: -local.y };
}

function clientToWorld(ev: PointerEvent | MouseEvent): { sM: number; zM: number } | null {
  const d = clientToDisplay(ev);
  if (!d) return null;
  return fromDisplay(d.s, d.z);
}

function onCanvasClick(ev: MouseEvent) {
  if (dragging.value) return;
  if ((ev.target as Element).closest?.(".node")) return;
  const world = clientToWorld(ev);
  if (!world) return;

  if (props.tool === "intake") {
    const r = placeIntake(props.site, world);
    emit("update:site", r.site);
    emit("update:selection", r.selection);
    emit("update:tool", "select");
    return;
  }
  if (props.tool === "turbine") {
    const r = placeTurbine(props.site, world);
    emit("update:site", r.site);
    emit("update:selection", r.selection);
    emit("update:tool", "select");
    return;
  }
  if (props.tool === "bend") {
    if (!props.site.intake) return;
    const r = addBend(props.site, world);
    emit("update:site", r.site);
    emit("update:selection", r.selection);
    return;
  }
  emit("update:selection", null);
}

function selectElement(sel: SiteSelection, ev: MouseEvent) {
  ev.stopPropagation();
  emit("update:selection", sel);
  emit("update:tool", "select");
}

function onPointerDown(sel: SiteSelection, ev: PointerEvent) {
  ev.stopPropagation();
  ev.preventDefault();
  emit("update:selection", sel);
  emit("update:tool", "select");
  dragging.value = sel;
  dragElevOrigin.value = elevOriginLive.value;
  svgRef.value?.setPointerCapture?.(ev.pointerId);
}

function onPointerMove(ev: PointerEvent) {
  if (!dragging.value) return;
  const world = clientToWorld(ev);
  if (!world) return;
  const sel = dragging.value;
  const site = props.site;
  if (sel.kind === "intake" && site.intake) {
    emit("update:site", { ...site, intake: world });
  } else if (sel.kind === "turbine" && site.turbine) {
    // Moving turbine changes elev origin; store new absolute z from display.
    emit("update:site", { ...site, turbine: world });
  } else if (sel.kind === "bend") {
    const bends = site.bends.map((b, i) => (i === sel.index ? world : b));
    emit("update:site", { ...site, bends });
  }
}

function onPointerUp(ev: PointerEvent) {
  if (!dragging.value) return;
  dragging.value = null;
  dragElevOrigin.value = null;
  try {
    svgRef.value?.releasePointerCapture?.(ev.pointerId);
  } catch {
    /* ignore */
  }
  if (props.site.bends.length > 1) {
    const sorted = [...props.site.bends].sort((a, b) => a.sM - b.sM);
    emit("update:site", { ...props.site, bends: sorted });
  }
}

const polyline = computed(() => {
  const pts = profilePoints(props.site);
  if (pts.length < 2) return "";
  return pts
    .map((p) => {
      const d = toDisplay(p);
      return `${d.s},${-d.z}`;
    })
    .join(" ");
});

const status = computed(() => {
  const base = siteStatusMessage(props.site);
  if (!triangle.value) return base;
  const t = triangle.value;
  const headNote =
    t.headM >= 0
      ? `H=${t.headM.toFixed(1)} m head`
      : `H=${Math.abs(t.headM).toFixed(1)} m (uphill — no head)`;
  return `${base} · horizontal run ${t.runM.toFixed(1)} m · ${headNote} · pipe L=${t.pipeM.toFixed(1)} m`;
});

const tools: { id: ToolId; label: string; hint: string }[] = [
  { id: "select", label: "Select", hint: "Select and drag elements" },
  { id: "intake", label: "Intake", hint: "Place or move intake" },
  { id: "bend", label: "Penstock bend", hint: "Add a bend (after intake)" },
  { id: "turbine", label: "Turbine", hint: "Place powerhouse / turbine" },
];

function isSelected(sel: SiteSelection): boolean {
  const cur = props.selection;
  if (!cur || !sel) return false;
  if (cur.kind !== sel.kind) return false;
  if (cur.kind === "bend" && sel.kind === "bend") return cur.index === sel.index;
  return true;
}

function setTool(id: ToolId) {
  emit("update:tool", id);
}

function resetView() {
  view.value = { ...DEFAULT_VIEW };
}

const elevAxisTitle = computed(() =>
  props.site.turbine ? "elevation above turbine (m)" : "elevation (m)",
);
</script>

<template>
  <section class="canvas" aria-label="Site canvas">
    <header class="toolbar" role="toolbar" aria-label="Construction tools">
      <button
        v-for="t in tools"
        :key="t.id"
        type="button"
        class="tool"
        :class="{ active: tool === t.id }"
        :title="t.hint"
        :disabled="t.id === 'bend' && !site.intake"
        @click="setTool(t.id)"
      >
        {{ t.label }}
      </button>
      <label class="snap" title="Snap placement and drag to 10 m grid">
        <input v-model="snapToGrid" type="checkbox" />
        Snap to {{ GRID_STEP_M }} m
      </label>
      <button type="button" class="tool ghost" title="Reset view window" @click="resetView">
        Reset view
      </button>
    </header>

    <div class="plot-wrap">
      <svg
        ref="svgRef"
        class="plot"
        :viewBox="viewBox"
        preserveAspectRatio="xMidYMid meet"
        @click="onCanvasClick"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
      >
        <rect
          :x="view.sMin"
          :y="-view.zMax"
          :width="view.sMax - view.sMin"
          :height="view.zMax - view.zMin"
          class="plot-bg"
        />

        <!-- 10×10 m square grid (no tick numbers) -->
        <g class="grid-lines">
          <line
            v-for="s in sGrid"
            :key="'vs' + s"
            :x1="s"
            :x2="s"
            :y1="-view.zMin"
            :y2="-view.zMax"
            class="grid"
            :class="{ major: s === 0 }"
          />
          <line
            v-for="z in zGrid"
            :key="'hz' + z"
            :x1="view.sMin"
            :x2="view.sMax"
            :y1="-z"
            :y2="-z"
            class="grid"
            :class="{ major: z === 0 }"
          />
        </g>

        <!-- Emphasize elevation 0 (turbine datum when placed) -->
        <line
          class="datum"
          :x1="view.sMin"
          :x2="view.sMax"
          y1="0"
          y2="0"
        />

        <!-- Axis titles only (no numeric tick labels) -->
        <text
          :x="(view.sMin + view.sMax) / 2"
          :y="-view.zMin + fontM * 1.8"
          text-anchor="middle"
          :font-size="fontM * 0.95"
          class="axis-title"
        >
          horizontal distance (m)
        </text>
        <text
          :x="view.sMin + fontM * 1.5"
          :y="-(view.zMin + view.zMax) / 2"
          text-anchor="middle"
          :font-size="fontM * 0.95"
          class="axis-title"
          :transform="`rotate(-90, ${view.sMin + fontM * 1.5}, ${-((view.zMin + view.zMax) / 2)})`"
        >
          {{ elevAxisTitle }}
        </text>

        <!-- Teaching triangle: H (vertical), horizontal run, dashed straight-pipe L -->
        <g v-if="triangle" class="triangle-guide">
          <line
            :x1="triangle.intakeD.s"
            :y1="-triangle.intakeD.z"
            :x2="triangle.corner.s"
            :y2="-triangle.corner.z"
            class="tri-leg"
          />
          <line
            :x1="triangle.corner.s"
            :y1="-triangle.corner.z"
            :x2="triangle.turbineD.s"
            :y2="-triangle.turbineD.z"
            class="tri-leg"
          />
          <line
            :x1="triangle.intakeD.s"
            :y1="-triangle.intakeD.z"
            :x2="triangle.turbineD.s"
            :y2="-triangle.turbineD.z"
            class="tri-hyp"
          />
          <text
            :x="triangle.intakeD.s - fontM * 0.4"
            :y="-(triangle.intakeD.z + triangle.turbineD.z) / 2"
            text-anchor="end"
            :font-size="fontM * 0.9"
            class="tri-label"
          >
            H {{ triangle.headM.toFixed(1) }} m
          </text>
          <text
            :x="(triangle.intakeD.s + triangle.turbineD.s) / 2"
            :y="-triangle.turbineD.z + fontM * 1.2"
            text-anchor="middle"
            :font-size="fontM * 0.9"
            class="tri-label"
          >
            run {{ triangle.runM.toFixed(1) }} m
          </text>
          <text
            :x="(triangle.intakeD.s + triangle.turbineD.s) / 2 + fontM"
            :y="-(triangle.intakeD.z + triangle.turbineD.z) / 2 - fontM * 0.5"
            text-anchor="start"
            :font-size="fontM * 0.9"
            class="tri-label hyp"
          >
            pipe L {{ triangle.pipeM.toFixed(1) }} m
          </text>
        </g>

        <polyline v-if="polyline" :points="polyline" class="penstock" fill="none" />

        <!-- Intake -->
        <g
          v-if="site.intake"
          class="node intake"
          :class="{ selected: isSelected({ kind: 'intake' }) }"
          @click="selectElement({ kind: 'intake' }, $event)"
          @pointerdown="onPointerDown({ kind: 'intake' }, $event)"
        >
          <circle
            :cx="toDisplay(site.intake).s"
            :cy="-toDisplay(site.intake).z"
            :r="markR * 1.6"
            class="hit"
          />
          <circle
            :cx="toDisplay(site.intake).s"
            :cy="-toDisplay(site.intake).z"
            :r="markR"
            class="mark"
          />
          <text
            :x="toDisplay(site.intake).s"
            :y="-toDisplay(site.intake).z - markR * 1.8"
            text-anchor="middle"
            :font-size="fontM"
            class="label"
          >
            Intake
          </text>
        </g>

        <!-- Bends -->
        <g
          v-for="(b, i) in site.bends"
          :key="'bend-' + i"
          class="node bend"
          :class="{ selected: isSelected({ kind: 'bend', index: i }) }"
          @click="selectElement({ kind: 'bend', index: i }, $event)"
          @pointerdown="onPointerDown({ kind: 'bend', index: i }, $event)"
        >
          <circle
            :cx="toDisplay(b).s"
            :cy="-toDisplay(b).z"
            :r="markR * 1.5"
            class="hit"
          />
          <circle
            :cx="toDisplay(b).s"
            :cy="-toDisplay(b).z"
            :r="markR * 0.85"
            class="mark"
          />
          <text
            :x="toDisplay(b).s"
            :y="-toDisplay(b).z - markR * 1.7"
            text-anchor="middle"
            :font-size="fontM * 0.9"
            class="label"
          >
            B{{ i + 1 }}
          </text>
        </g>

        <!-- Turbine (sits on elevation datum when present) -->
        <g
          v-if="site.turbine"
          class="node turbine"
          :class="{ selected: isSelected({ kind: 'turbine' }) }"
          @click="selectElement({ kind: 'turbine' }, $event)"
          @pointerdown="onPointerDown({ kind: 'turbine' }, $event)"
        >
          <circle
            :cx="toDisplay(site.turbine).s"
            :cy="-toDisplay(site.turbine).z"
            :r="markR * 1.6"
            class="hit"
          />
          <rect
            :x="toDisplay(site.turbine).s - markR"
            :y="-toDisplay(site.turbine).z - markR"
            :width="markR * 2"
            :height="markR * 2"
            class="mark-sq"
          />
          <text
            :x="toDisplay(site.turbine).s"
            :y="-toDisplay(site.turbine).z - markR * 1.8"
            text-anchor="middle"
            :font-size="fontM"
            class="label"
          >
            Turbine
          </text>
        </g>
      </svg>

      <div v-if="!site.intake && !site.turbine && site.bends.length === 0" class="empty-overlay">
        <p class="title">Clean slate</p>
        <p>
          Place an <strong>intake</strong> upslope, then a <strong>turbine</strong> lower.
          Elevation 0 is the turbine once placed — head is the vertical drop.
        </p>
        <p class="muted">
          Axes: horizontal distance × elevation. Pipe length L follows the penstock (slope /
          bends), not the horizontal axis.
        </p>
      </div>
    </div>

    <p class="status">
      {{ status }}
      <span v-if="isSiteComplete(site) && triangle" class="geom">
        · slope
        {{
          (
            (Math.atan2(Math.max(0, triangle.headM), Math.max(1e-6, triangle.runM)) * 180) /
            Math.PI
          ).toFixed(1)
        }}°
      </span>
      <span class="geom"> · grid {{ GRID_STEP_M }} m</span>
    </p>
  </section>
</template>

<style scoped>
.canvas {
  display: flex;
  flex-direction: column;
  min-height: 24rem;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--panel);
  overflow: hidden;
}

.toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  align-items: center;
  padding: 0.55rem 0.7rem;
  border-bottom: 1px solid var(--border);
  background: var(--toolbar);
}

.tool {
  font: inherit;
  font-size: 0.8rem;
  padding: 0.3rem 0.65rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--fg);
  cursor: pointer;
}

.tool:hover:not(:disabled) {
  border-color: var(--accent);
}

.tool.active {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}

.tool.ghost {
  margin-left: auto;
  opacity: 0.85;
}

.tool:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.snap {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.8rem;
  color: var(--fg);
  margin-left: 0.35rem;
  user-select: none;
}

.plot-wrap {
  position: relative;
  flex: 1;
  min-height: 20rem;
  background: var(--canvas-bg);
}

.plot {
  width: 100%;
  height: 100%;
  min-height: 20rem;
  display: block;
  cursor: crosshair;
  color: var(--fg);
  touch-action: none;
  user-select: none;
}

.plot-bg {
  fill: var(--canvas-bg);
}

.grid {
  stroke: var(--fg);
  stroke-opacity: 0.12;
  stroke-width: 0.35;
}

.grid.major {
  stroke-opacity: 0.28;
  stroke-width: 0.55;
}

.datum {
  stroke: var(--fg);
  stroke-opacity: 0.45;
  stroke-width: 0.7;
}

.axis-title {
  fill: var(--muted-fg);
  pointer-events: none;
}

.tri-leg {
  stroke: var(--muted-fg);
  stroke-opacity: 0.55;
  stroke-width: 0.7;
  stroke-dasharray: 2 1.5;
}

.tri-hyp {
  stroke: #8a6d3b;
  stroke-opacity: 0.55;
  stroke-width: 0.7;
  stroke-dasharray: 3 2;
}

.tri-label {
  fill: var(--muted-fg);
  pointer-events: none;
}

.tri-label.hyp {
  fill: #8a6d3b;
}

.penstock {
  stroke: var(--accent);
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
  opacity: 0.95;
}

.node {
  cursor: grab;
}

.node:active {
  cursor: grabbing;
}

.hit {
  fill: transparent;
  stroke: transparent;
}

.mark {
  fill: var(--panel);
  stroke: var(--accent);
  stroke-width: 0.7;
}

.mark-sq {
  fill: var(--panel);
  stroke: #c47b2b;
  stroke-width: 0.7;
}

.node.intake .mark {
  stroke: #2a7a4b;
  fill: color-mix(in srgb, #2a7a4b 30%, var(--panel));
}

.node.turbine .mark-sq {
  fill: color-mix(in srgb, #c47b2b 30%, var(--panel));
}

.node.selected .mark,
.node.selected .mark-sq {
  stroke-width: 1.2;
}

.node.selected .hit {
  stroke: var(--accent);
  stroke-opacity: 0.3;
  stroke-width: 1;
}

.label {
  fill: var(--fg);
  pointer-events: none;
  user-select: none;
  font-weight: 600;
}

.empty-overlay {
  position: absolute;
  inset: 2.5rem 2rem 2.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: 0.45rem;
  padding: 1.25rem;
  border: 1px dashed var(--border-strong);
  border-radius: 12px;
  background: color-mix(in srgb, var(--panel) 90%, transparent);
  max-width: 30rem;
  margin: auto;
  pointer-events: none;
}

.empty-overlay .title {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 650;
}

.empty-overlay p {
  margin: 0;
  max-width: 26rem;
  line-height: 1.45;
  font-size: 0.9rem;
  color: var(--fg);
}

.muted {
  color: var(--muted-fg) !important;
  font-size: 0.78rem !important;
}

.status {
  margin: 0;
  padding: 0.45rem 0.75rem;
  font-size: 0.8rem;
  color: var(--muted-fg);
  border-top: 1px solid var(--border);
  background: var(--toolbar);
}

.geom {
  color: var(--fg);
}
</style>
