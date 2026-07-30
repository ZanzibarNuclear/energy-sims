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
  markerRadiusM,
  tickStep,
  ticks,
  toSvgViewBox,
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

/** Fixed world window so dragging moves the point on screen (not the camera). */
const view = ref<WorldBounds>({ ...DEFAULT_VIEW });

const viewBox = computed(() => toSvgViewBox(view.value));
const markR = computed(() => markerRadiusM(view.value));

const sTicks = computed(() => {
  const b = view.value;
  return ticks(b.sMin, b.sMax, tickStep(b.sMax - b.sMin));
});
const zTicks = computed(() => {
  const b = view.value;
  return ticks(b.zMin, b.zMax, tickStep(b.zMax - b.zMin));
});

/** Head / run / pipe length for triangle teaching graphic. */
const triangle = computed(() => {
  if (!props.site.intake || !props.site.turbine) return null;
  const a = props.site.intake;
  const t = props.site.turbine;
  const runM = Math.abs(t.sM - a.sM);
  const headM = a.zM - t.zM;
  const pts = profilePoints(props.site);
  let pipeM = 0;
  for (let i = 1; i < pts.length; i++) {
    const p0 = pts[i - 1]!;
    const p1 = pts[i]!;
    pipeM += Math.hypot(p1.sM - p0.sM, p1.zM - p0.zM);
  }
  // Right angle corner of the elevation triangle (vertical under intake, horizontal to turbine s).
  const corner = { sM: a.sM, zM: t.zM };
  return {
    intake: a,
    turbine: t,
    corner,
    runM,
    headM,
    pipeM,
    steep: headM > 0 && runM > 1e-6 ? headM / runM : 0,
  };
});

function clientToWorld(ev: PointerEvent | MouseEvent): { sM: number; zM: number } | null {
  const svg = svgRef.value;
  if (!svg) return null;
  const pt = svg.createSVGPoint();
  pt.x = ev.clientX;
  pt.y = ev.clientY;
  const ctm = svg.getScreenCTM();
  if (!ctm) return null;
  const local = pt.matrixTransform(ctm.inverse());
  return clampPoint(local.x, -local.y, view.value);
}

function onCanvasClick(ev: MouseEvent) {
  if (dragging.value) return;
  // Ignore clicks that started on a node (pointerdown already handled).
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
    emit("update:site", { ...site, turbine: world });
  } else if (sel.kind === "bend") {
    const bends = site.bends.map((b, i) => (i === sel.index ? world : b));
    emit("update:site", { ...site, bends });
  }
}

function onPointerUp(ev: PointerEvent) {
  if (!dragging.value) return;
  dragging.value = null;
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
  return pts.map((p) => `${p.sM},${-p.zM}`).join(" ");
});

const status = computed(() => {
  const base = siteStatusMessage(props.site);
  if (!triangle.value) return base;
  const t = triangle.value;
  const headNote =
    t.headM >= 0
      ? `H=${t.headM.toFixed(1)} m drop`
      : `H=${Math.abs(t.headM).toFixed(1)} m (uphill — no head)`;
  return `${base} · run ${t.runM.toFixed(1)} m · ${headNote} · pipe L=${t.pipeM.toFixed(1)} m`;
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

/** Font size in world units for readable labels. */
const fontM = computed(() => Math.max(3.2, viewWidthSafe() * 0.018));
function viewWidthSafe() {
  return view.value.sMax - view.value.sMin;
}
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
      <button type="button" class="tool ghost" title="Reset view to default window" @click="resetView">
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
        <!-- Plot background -->
        <rect
          :x="view.sMin"
          :y="-view.zMax"
          :width="view.sMax - view.sMin"
          :height="view.zMax - view.zMin"
          class="plot-bg"
        />

        <!-- Vertical grid + s labels -->
        <g class="grid-lines">
          <line
            v-for="s in sTicks"
            :key="'vs' + s"
            :x1="s"
            :x2="s"
            :y1="-view.zMin"
            :y2="-view.zMax"
            class="grid"
          />
          <line
            v-for="z in zTicks"
            :key="'hz' + z"
            :x1="view.sMin"
            :x2="view.sMax"
            :y1="-z"
            :y2="-z"
            class="grid"
          />
        </g>

        <!-- Axes through origin if visible, else along plot edge -->
        <line
          class="axis-line"
          :x1="view.sMin"
          :x2="view.sMax"
          :y1="0"
          :y2="0"
        />
        <line
          class="axis-line"
          :x1="0"
          :x2="0"
          :y1="-view.zMin"
          :y2="-view.zMax"
        />

        <!-- Tick labels (s along bottom of view, z along left) -->
        <g class="tick-labels">
          <text
            v-for="s in sTicks"
            :key="'sl' + s"
            :x="s"
            :y="-view.zMin + fontM * 1.15"
            text-anchor="middle"
            :font-size="fontM * 0.85"
            class="tick"
          >
            {{ s }}
          </text>
          <text
            v-for="z in zTicks"
            :key="'zl' + z"
            :x="view.sMin + fontM * 0.35"
            :y="-z + fontM * 0.3"
            text-anchor="start"
            :font-size="fontM * 0.85"
            class="tick"
          >
            {{ z }}
          </text>
        </g>

        <text
          :x="(view.sMin + view.sMax) / 2"
          :y="-view.zMin + fontM * 2.4"
          text-anchor="middle"
          :font-size="fontM * 0.9"
          class="axis-title"
        >
          ground distance s (m)
        </text>
        <text
          :x="view.sMin + fontM * 1.6"
          :y="-(view.zMin + view.zMax) / 2"
          text-anchor="middle"
          :font-size="fontM * 0.9"
          class="axis-title"
          :transform="`rotate(-90, ${view.sMin + fontM * 1.6}, ${-((view.zMin + view.zMax) / 2)})`"
        >
          elevation z (m)
        </text>

        <!-- Right triangle guide: vertical head + horizontal run + hypotenuse = penstock intent -->
        <g v-if="triangle" class="triangle-guide">
          <!-- vertical drop under intake -->
          <line
            :x1="triangle.intake.sM"
            :y1="-triangle.intake.zM"
            :x2="triangle.corner.sM"
            :y2="-triangle.corner.zM"
            class="tri-leg"
          />
          <!-- horizontal run at turbine elevation -->
          <line
            :x1="triangle.corner.sM"
            :y1="-triangle.corner.zM"
            :x2="triangle.turbine.sM"
            :y2="-triangle.turbine.zM"
            class="tri-leg"
          />
          <!-- dashed ideal hypotenuse intake → turbine (straight pipe) -->
          <line
            :x1="triangle.intake.sM"
            :y1="-triangle.intake.zM"
            :x2="triangle.turbine.sM"
            :y2="-triangle.turbine.zM"
            class="tri-hyp"
          />
          <text
            :x="triangle.intake.sM - fontM * 0.4"
            :y="-(triangle.intake.zM + triangle.turbine.zM) / 2"
            text-anchor="end"
            :font-size="fontM * 0.9"
            class="tri-label"
          >
            H {{ triangle.headM.toFixed(1) }} m
          </text>
          <text
            :x="(triangle.intake.sM + triangle.turbine.sM) / 2"
            :y="-triangle.turbine.zM + fontM * 1.2"
            text-anchor="middle"
            :font-size="fontM * 0.9"
            class="tri-label"
          >
            run {{ triangle.runM.toFixed(1) }} m
          </text>
          <text
            :x="(triangle.intake.sM + triangle.turbine.sM) / 2 + fontM"
            :y="-(triangle.intake.zM + triangle.turbine.zM) / 2 - fontM * 0.5"
            text-anchor="start"
            :font-size="fontM * 0.9"
            class="tri-label hyp"
          >
            L {{ triangle.pipeM.toFixed(1) }} m
          </text>
        </g>

        <!-- Actual penstock path (may include bends) -->
        <polyline
          v-if="polyline"
          :points="polyline"
          class="penstock"
          fill="none"
        />

        <!-- Intake -->
        <g
          v-if="site.intake"
          class="node intake"
          :class="{ selected: isSelected({ kind: 'intake' }) }"
          @click="selectElement({ kind: 'intake' }, $event)"
          @pointerdown="onPointerDown({ kind: 'intake' }, $event)"
        >
          <circle :cx="site.intake.sM" :cy="-site.intake.zM" :r="markR * 1.6" class="hit" />
          <circle :cx="site.intake.sM" :cy="-site.intake.zM" :r="markR" class="mark" />
          <text
            :x="site.intake.sM"
            :y="-site.intake.zM - markR * 1.8"
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
          <circle :cx="b.sM" :cy="-b.zM" :r="markR * 1.5" class="hit" />
          <circle :cx="b.sM" :cy="-b.zM" :r="markR * 0.85" class="mark" />
          <text
            :x="b.sM"
            :y="-b.zM - markR * 1.7"
            text-anchor="middle"
            :font-size="fontM * 0.9"
            class="label"
          >
            B{{ i + 1 }}
          </text>
        </g>

        <!-- Turbine -->
        <g
          v-if="site.turbine"
          class="node turbine"
          :class="{ selected: isSelected({ kind: 'turbine' }) }"
          @click="selectElement({ kind: 'turbine' }, $event)"
          @pointerdown="onPointerDown({ kind: 'turbine' }, $event)"
        >
          <circle :cx="site.turbine.sM" :cy="-site.turbine.zM" :r="markR * 1.6" class="hit" />
          <rect
            :x="site.turbine.sM - markR"
            :y="-site.turbine.zM - markR"
            :width="markR * 2"
            :height="markR * 2"
            class="mark-sq"
          />
          <text
            :x="site.turbine.sM"
            :y="-site.turbine.zM - markR * 1.8"
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
          Place an <strong>intake</strong> high on the slope, then a
          <strong>turbine</strong> lower and farther along ground distance.
        </p>
        <p class="muted">
          Grid is meters (s, z). Bigger elevation drop for a given run → steeper penstock.
        </p>
      </div>
    </div>

    <p class="status">
      {{ status }}
      <span v-if="isSiteComplete(site) && triangle" class="geom">
        · slope {{ (Math.atan2(Math.max(0, triangle.headM), Math.max(1e-6, triangle.runM)) * 180 / Math.PI).toFixed(1) }}°
      </span>
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
  stroke-opacity: 0.1;
  stroke-width: 0.35;
}

.axis-line {
  stroke: var(--fg);
  stroke-opacity: 0.35;
  stroke-width: 0.5;
}

.tick {
  fill: var(--muted-fg);
  pointer-events: none;
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
  max-width: 28rem;
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
  max-width: 24rem;
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
