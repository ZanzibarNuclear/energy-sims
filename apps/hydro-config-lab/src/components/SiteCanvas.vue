<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  addBendOnPenstock,
  constrainBendPoint,
  deleteBend,
  moveBend,
  moveIntake,
  moveTurbine,
  profilePoints,
  segments,
  validatePenstock,
  type Site,
  type SiteSelection,
} from "../lib/site";
import {
  clampPoint,
  DEFAULT_VIEW,
  displayToWorld,
  elevationOriginZ,
  FIT_MARGIN_M,
  fitViewToPoints,
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
}>();

const emit = defineEmits<{
  "update:site": [Site];
  "update:selection": [SiteSelection];
}>();

type DragTarget =
  | { kind: "intake" }
  | { kind: "turbine" }
  | { kind: "bend"; index: number };

const svgRef = ref<SVGSVGElement | null>(null);
const dragging = ref<DragTarget | null>(null);
const dragElevOrigin = ref<number | null>(null);
/** After interacting with a node, ignore the synthetic canvas click that would clear selection. */
const suppressCanvasClick = ref(false);
const snapToGrid = ref(true);
const view = ref<WorldBounds>({ ...DEFAULT_VIEW });
const clampHint = ref("");

const elevOriginLive = computed(() => elevationOriginZ(props.site.turbine.zM));
const elevOrigin = computed(() =>
  dragElevOrigin.value != null ? dragElevOrigin.value : elevOriginLive.value,
);

const penstockIssues = computed(() => validatePenstock(props.site));

function refitView() {
  view.value = fitViewToPoints(
    profilePoints(props.site),
    elevationOriginZ(props.site.turbine.zM),
    FIT_MARGIN_M,
  );
}

// Auto-fit when layout changes (not mid-drag — refit on release instead).
watch(
  () => props.site,
  () => {
    if (!dragging.value) refitView();
  },
  { deep: true, immediate: true },
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

const segs = computed(() => segments(props.site));

const polyline = computed(() =>
  profilePoints(props.site)
    .map((p) => {
      const d = toDisplay(p);
      return `${d.s},${-d.z}`;
    })
    .join(" "),
);

/** Midpoint labels for slope on each segment (display coords). */
const segmentLabels = computed(() =>
  segs.value.map((seg) => {
    const a = toDisplay(seg.from);
    const b = toDisplay(seg.to);
    return {
      index: seg.index,
      s: (a.s + b.s) / 2,
      z: (a.z + b.z) / 2,
      slopeDeg: seg.slopeDeg,
      lengthM: seg.lengthM,
    };
  }),
);

const triangle = computed(() => {
  const a = props.site.intake;
  const t = props.site.turbine;
  const aD = toDisplay(a);
  const tD = toDisplay(t);
  const runM = Math.abs(t.sM - a.sM);
  const headM = a.zM - t.zM;
  let pipeM = 0;
  for (const s of segs.value) pipeM += s.lengthM;
  return {
    intakeD: aD,
    turbineD: tD,
    corner: { s: aD.s, z: tD.z },
    runM,
    headM,
    pipeM,
  };
});

/**
 * Label position for total pipe L: midpoint along the actual penstock path,
 * offset perpendicular (upslope / "above" the line) so it tracks bends.
 */
const pipeLengthLabel = computed(() => {
  const list = segs.value;
  if (list.length === 0) return null;
  const total = list.reduce((sum, s) => sum + s.lengthM, 0);
  if (total < 1e-9) return null;

  let remaining = total / 2;
  let from = list[0]!.from;
  let to = list[0]!.to;
  for (const seg of list) {
    if (remaining <= seg.lengthM) {
      from = seg.from;
      to = seg.to;
      break;
    }
    remaining -= seg.lengthM;
  }
  const segLen = Math.hypot(to.sM - from.sM, to.zM - from.zM);
  const t = segLen > 1e-9 ? remaining / segLen : 0.5;
  const mid = {
    sM: from.sM + (to.sM - from.sM) * t,
    zM: from.zM + (to.zM - from.zM) * t,
  };
  const d = toDisplay(mid);

  // Unit direction of this segment in display space; offset perpendicular.
  const a = toDisplay(from);
  const b = toDisplay(to);
  let ds = b.s - a.s;
  let dz = b.z - a.z;
  const n = Math.hypot(ds, dz) || 1;
  ds /= n;
  dz /= n;
  // Perpendicular in (s, z): (-dz, ds). Prefer the side with higher elevation
  // (away from the downhill side) so the label sits in open air.
  let ps = -dz;
  let pz = ds;
  if (pz < 0) {
    ps = -ps;
    pz = -pz;
  }
  // Offset from the path — clear of slope labels but not too far out.
  const offset = Math.max(fontM.value * 2.2, 10);
  return {
    s: d.s + ps * offset,
    z: d.z + pz * offset,
    pipeM: total,
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
  return fromDisplay(local.x, -local.y);
}

function onAddBend() {
  const r = addBendOnPenstock(props.site);
  // Optionally snap midpoint to grid
  if (snapToGrid.value && r.selection) {
    const b = r.site.bends[r.selection.index];
    if (b) {
      const snapped = snapPoint(b.sM, b.zM, GRID_STEP_M);
      r.site.bends[r.selection.index] = snapped;
      r.site.bends.sort((x, y) => x.sM - y.sM);
      const idx = r.site.bends.findIndex(
        (p) => p.sM === snapped.sM && p.zM === snapped.zM,
      );
      r.selection = { kind: "bend", index: idx >= 0 ? idx : r.selection.index };
    }
  }
  emit("update:site", r.site);
  emit("update:selection", r.selection);
}

function onDeleteSelectedBend() {
  if (!props.selection) return;
  emit("update:site", deleteBend(props.site, props.selection.index));
  emit("update:selection", null);
}

function onPointerDown(target: DragTarget, ev: PointerEvent) {
  ev.stopPropagation();
  ev.preventDefault();
  // Pointer capture makes the following click land on the SVG, which would
  // otherwise clear bend selection immediately on mouse-up.
  suppressCanvasClick.value = true;
  if (target.kind === "bend") {
    emit("update:selection", { kind: "bend", index: target.index });
  } else {
    emit("update:selection", null);
  }
  dragging.value = target;
  dragElevOrigin.value = elevOriginLive.value;
  svgRef.value?.setPointerCapture?.(ev.pointerId);
}

function noteClamp(requested: { sM: number; zM: number }, applied: { sM: number; zM: number }) {
  const elevBlocked = Math.abs(requested.zM - applied.zM) > 0.05;
  const sBlocked = Math.abs(requested.sM - applied.sM) > 0.05;
  if (!elevBlocked && !sBlocked) {
    clampHint.value = "";
    return;
  }
  clampHint.value =
    "Penstock tip: keep the pipe downhill (or flat) from intake to turbine. " +
    "Bends cannot rise above the previous point or the intake, and nothing can sit below the turbine — " +
    "an uphill pocket traps air and stalls gravity flow.";
}

function onPointerMove(ev: PointerEvent) {
  if (!dragging.value) return;
  const world = clientToWorld(ev);
  if (!world) return;
  const t = dragging.value;
  if (t.kind === "intake") {
    const site = moveIntake(props.site, world);
    noteClamp(world, site.intake);
    emit("update:site", site);
  } else if (t.kind === "turbine") {
    const site = moveTurbine(props.site, world);
    noteClamp(world, site.turbine);
    emit("update:site", site);
  } else {
    const applied = constrainBendPoint(props.site, t.index, world);
    noteClamp(world, applied);
    const site = moveBend(props.site, t.index, world);
    const idx = site.bends.findIndex(
      (b) => Math.abs(b.sM - applied.sM) < 1e-6 && Math.abs(b.zM - applied.zM) < 1e-6,
    );
    emit("update:site", site);
    if (idx >= 0) {
      dragging.value = { kind: "bend", index: idx };
      emit("update:selection", { kind: "bend", index: idx });
    }
  }
}

function onPointerUp(ev: PointerEvent) {
  if (!dragging.value) return;
  const ended = dragging.value;
  dragging.value = null;
  dragElevOrigin.value = null;
  // Keep bend selected after click/drag so Delete bend stays available.
  if (ended.kind === "bend") {
    emit("update:selection", { kind: "bend", index: ended.index });
  }
  try {
    svgRef.value?.releasePointerCapture?.(ev.pointerId);
  } catch {
    /* ignore */
  }
  refitView();
}

function onCanvasClick(ev: MouseEvent) {
  if (suppressCanvasClick.value) {
    suppressCanvasClick.value = false;
    return;
  }
  if (dragging.value) return;
  if ((ev.target as Element).closest?.(".node")) return;
  emit("update:selection", null);
}

function isBendSelected(i: number): boolean {
  return props.selection?.kind === "bend" && props.selection.index === i;
}
</script>

<template>
  <section class="canvas" aria-label="Site layout">
    <header class="toolbar">
      <label class="snap" title="Snap drag to 10 m grid">
        <input v-model="snapToGrid" type="checkbox" />
        Snap to {{ GRID_STEP_M }} m
      </label>
      <button type="button" class="tool primary" @click="onAddBend">Add a bend</button>
      <button
        v-if="selection"
        type="button"
        class="tool danger"
        @click="onDeleteSelectedBend"
      >
        Delete bend
      </button>
    </header>

    <p v-if="clampHint || penstockIssues.length" class="edu" role="status">
      {{ clampHint || penstockIssues[0]?.message }}
    </p>

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

        <line class="datum" :x1="view.sMin" :x2="view.sMax" y1="0" y2="0" />

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
          elevation above turbine (m)
        </text>

        <!-- Head / run / total pipe L guide -->
        <g class="triangle-guide">
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
          <text
            :x="triangle.intakeD.s - fontM * 0.4"
            :y="-(triangle.intakeD.z + triangle.turbineD.z) / 2"
            text-anchor="end"
            :font-size="fontM * 0.9"
            class="tri-label"
          >
            H {{ triangle.headM.toFixed(0) }} m
          </text>
          <text
            :x="(triangle.intakeD.s + triangle.turbineD.s) / 2"
            :y="-triangle.turbineD.z + fontM * 1.2"
            text-anchor="middle"
            :font-size="fontM * 0.9"
            class="tri-label"
          >
            run {{ triangle.runM.toFixed(0) }} m
          </text>
        </g>

        <polyline :points="polyline" class="penstock" fill="none" />

        <!-- Pipe L tracks midpoint of the real penstock path -->
        <text
          v-if="pipeLengthLabel"
          :x="pipeLengthLabel.s"
          :y="-pipeLengthLabel.z"
          text-anchor="middle"
          :font-size="fontM * 0.95"
          class="pipe-l-label"
        >
          Pipe L {{ pipeLengthLabel.pipeM.toFixed(0) }} m
        </text>

        <!-- Slope angle on each segment -->
        <text
          v-for="lab in segmentLabels"
          :key="'sl' + lab.index"
          :x="lab.s"
          :y="-lab.z - fontM * 0.35"
          text-anchor="middle"
          :font-size="fontM * 0.95"
          class="slope-label"
        >
          {{ lab.slopeDeg.toFixed(0) }}°
        </text>

        <!-- Intake (draggable, not deletable) -->
        <g
          class="node intake"
          @pointerdown="onPointerDown({ kind: 'intake' }, $event)"
          @click.stop
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

        <!-- Bends (selectable + deletable) -->
        <g
          v-for="(b, i) in site.bends"
          :key="'bend-' + i"
          class="node bend"
          :class="{ selected: isBendSelected(i) }"
          @pointerdown="onPointerDown({ kind: 'bend', index: i }, $event)"
          @click.stop
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

        <!-- Turbine -->
        <g
          class="node turbine"
          @pointerdown="onPointerDown({ kind: 'turbine' }, $event)"
          @click.stop
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
    </div>
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
  gap: 0.5rem;
  align-items: center;
  padding: 0.55rem 0.75rem;
  border-bottom: 1px solid var(--border);
  background: var(--toolbar);
}

.snap {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  font-size: 0.85rem;
  color: var(--fg);
  user-select: none;
}

.tool {
  font: inherit;
  font-size: 0.85rem;
  padding: 0.35rem 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--fg);
  cursor: pointer;
}

.tool:hover {
  border-color: var(--accent);
}

.tool.primary {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
  font-weight: 600;
}

.tool.primary:hover {
  filter: brightness(1.06);
}

.tool.danger {
  color: #c44;
  border-color: color-mix(in srgb, #c44 45%, var(--border));
  margin-left: auto;
}

.edu {
  margin: 0;
  padding: 0.5rem 0.75rem;
  font-size: 0.82rem;
  line-height: 1.4;
  color: var(--fg);
  background: color-mix(in srgb, #c9a227 16%, var(--toolbar));
  border-bottom: 1px solid color-mix(in srgb, #c9a227 35%, var(--border));
}

.plot-wrap {
  position: relative;
  flex: 1;
  min-height: 22rem;
  background: var(--canvas-bg);
}

.plot {
  width: 100%;
  height: 100%;
  min-height: 22rem;
  display: block;
  cursor: default;
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
  stroke-opacity: 0.5;
  stroke-width: 0.7;
  stroke-dasharray: 2 1.5;
}

.tri-label {
  fill: var(--muted-fg);
  pointer-events: none;
}

.penstock {
  stroke: var(--accent);
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.pipe-l-label {
  fill: var(--accent);
  font-weight: 700;
  pointer-events: none;
  paint-order: stroke;
  stroke: var(--canvas-bg);
  stroke-width: 0.85px;
}

.slope-label {
  fill: var(--fg);
  font-weight: 650;
  pointer-events: none;
  paint-order: stroke;
  stroke: var(--canvas-bg);
  stroke-width: 0.6px;
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

.node.bend.selected .mark {
  stroke-width: 1.3;
  stroke: var(--accent);
}

.node.bend.selected .hit {
  stroke: var(--accent);
  stroke-opacity: 0.35;
  stroke-width: 1;
}

.label {
  fill: var(--fg);
  pointer-events: none;
  user-select: none;
  font-weight: 600;
}
</style>
