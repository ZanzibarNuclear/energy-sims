<script setup lang="ts">
import { computed, ref } from "vue";
import {
  addBend,
  placeIntake,
  placeTurbine,
  profilePoints,
  siteStatusMessage,
  type Site,
  type SiteSelection,
  type ToolId,
} from "../lib/site";

const props = withDefaults(
  defineProps<{
    site: Site;
    selection: SiteSelection;
    tool: ToolId;
    /** World bounds (meters). Auto-expands when points go outside. */
    viewPaddingM?: number;
  }>(),
  { viewPaddingM: 20 },
);

const emit = defineEmits<{
  "update:site": [Site];
  "update:selection": [SiteSelection];
  "update:tool": [ToolId];
}>();

const svgRef = ref<SVGSVGElement | null>(null);
const dragging = ref<SiteSelection>(null);

const defaultBounds = { sMin: 0, sMax: 200, zMin: 50, zMax: 120 };

const bounds = computed(() => {
  const pts = profilePoints(props.site);
  if (pts.length === 0) return { ...defaultBounds };
  const pad = props.viewPaddingM;
  let sMin = Math.min(...pts.map((p) => p.sM));
  let sMax = Math.max(...pts.map((p) => p.sM));
  let zMin = Math.min(...pts.map((p) => p.zM));
  let zMax = Math.max(...pts.map((p) => p.zM));
  if (sMax - sMin < 40) {
    const mid = (sMin + sMax) / 2;
    sMin = mid - 20;
    sMax = mid + 20;
  }
  if (zMax - zMin < 20) {
    const mid = (zMin + zMax) / 2;
    zMin = mid - 10;
    zMax = mid + 10;
  }
  return {
    sMin: sMin - pad,
    sMax: sMax + pad,
    zMin: zMin - pad,
    zMax: zMax + pad,
  };
});

const viewBox = computed(() => {
  const b = bounds.value;
  const w = b.sMax - b.sMin;
  const h = b.zMax - b.zMin;
  // SVG y increases downward; flip elevation so higher z is higher on screen.
  return `${b.sMin} ${-b.zMax} ${w} ${h}`;
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
  return { sM: local.x, zM: -local.y };
}

function onCanvasClick(ev: MouseEvent) {
  if (dragging.value) return;
  const world = clientToWorld(ev);
  if (!world) return;
  const point = { sM: world.sM, zM: world.zM };

  if (props.tool === "intake") {
    const r = placeIntake(props.site, point);
    emit("update:site", r.site);
    emit("update:selection", r.selection);
    emit("update:tool", "select");
    return;
  }
  if (props.tool === "turbine") {
    const r = placeTurbine(props.site, point);
    emit("update:site", r.site);
    emit("update:selection", r.selection);
    emit("update:tool", "select");
    return;
  }
  if (props.tool === "bend") {
    if (!props.site.intake) return;
    const r = addBend(props.site, point);
    emit("update:site", r.site);
    emit("update:selection", r.selection);
    return;
  }
  // select tool: click empty deselects
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
  (ev.target as Element).setPointerCapture?.(ev.pointerId);
}

function onPointerMove(ev: PointerEvent) {
  if (!dragging.value) return;
  const world = clientToWorld(ev);
  if (!world) return;
  const sel = dragging.value;
  const site = props.site;
  if (sel.kind === "intake" && site.intake) {
    emit("update:site", { ...site, intake: { sM: world.sM, zM: world.zM } });
  } else if (sel.kind === "turbine" && site.turbine) {
    emit("update:site", { ...site, turbine: { sM: world.sM, zM: world.zM } });
  } else if (sel.kind === "bend") {
    const bends = site.bends.map((b, i) =>
      i === sel.index ? { sM: world.sM, zM: world.zM } : b,
    );
    emit("update:site", { ...site, bends });
  }
}

function onPointerUp(ev: PointerEvent) {
  if (!dragging.value) return;
  dragging.value = null;
  try {
    (ev.target as Element).releasePointerCapture?.(ev.pointerId);
  } catch {
    /* ignore */
  }
  // Re-sort bends after drag so path order stays by sM
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

const status = computed(() => siteStatusMessage(props.site));

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
        <!-- axis ticks (light) -->
        <defs>
          <pattern
            id="grid"
            width="20"
            height="20"
            patternUnits="userSpaceOnUse"
            :patternTransform="`translate(${bounds.sMin}, ${-bounds.zMax})`"
          >
            <path
              d="M 20 0 L 0 0 0 20"
              fill="none"
              stroke="currentColor"
              stroke-opacity="0.08"
              stroke-width="0.4"
              vector-effect="non-scaling-stroke"
            />
          </pattern>
        </defs>
        <rect
          :x="bounds.sMin"
          :y="-bounds.zMax"
          :width="bounds.sMax - bounds.sMin"
          :height="bounds.zMax - bounds.zMin"
          fill="url(#grid)"
          class="grid-fill"
        />

        <polyline
          v-if="polyline"
          :points="polyline"
          class="penstock"
          fill="none"
          vector-effect="non-scaling-stroke"
        />

        <!-- Intake -->
        <g
          v-if="site.intake"
          class="node intake"
          :class="{ selected: isSelected({ kind: 'intake' }) }"
          @click="selectElement({ kind: 'intake' }, $event)"
          @pointerdown="onPointerDown({ kind: 'intake' }, $event)"
        >
          <circle :cx="site.intake.sM" :cy="-site.intake.zM" r="2.2" class="hit" />
          <circle :cx="site.intake.sM" :cy="-site.intake.zM" r="1.4" class="mark" />
          <text
            :x="site.intake.sM"
            :y="-site.intake.zM - 2.4"
            text-anchor="middle"
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
          <circle :cx="b.sM" :cy="-b.zM" r="2" class="hit" />
          <circle :cx="b.sM" :cy="-b.zM" r="1.1" class="mark" />
          <text :x="b.sM" :y="-b.zM - 2.2" text-anchor="middle" class="label">
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
          <rect
            :x="site.turbine.sM - 1.5"
            :y="-site.turbine.zM - 1.5"
            width="3"
            height="3"
            class="mark-sq"
          />
          <circle :cx="site.turbine.sM" :cy="-site.turbine.zM" r="2.2" class="hit" />
          <text
            :x="site.turbine.sM"
            :y="-site.turbine.zM - 2.4"
            text-anchor="middle"
            class="label"
          >
            Turbine
          </text>
        </g>
      </svg>

      <div v-if="!site.intake && !site.turbine && site.bends.length === 0" class="empty-overlay">
        <p class="title">Clean slate</p>
        <p>Choose <strong>Intake</strong>, then click the canvas to place it.</p>
        <p class="muted">x → ground distance (m) · y → elevation (m)</p>
      </div>

      <div class="axis-hint y">elevation (m)</div>
      <div class="axis-hint x">distance along ground (m)</div>
    </div>

    <p class="status">{{ status }}</p>
  </section>
</template>

<style scoped>
.canvas {
  display: flex;
  flex-direction: column;
  min-height: 22rem;
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

.tool:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.plot-wrap {
  position: relative;
  flex: 1;
  min-height: 18rem;
  background: var(--canvas-bg);
}

.plot {
  width: 100%;
  height: 100%;
  min-height: 18rem;
  display: block;
  cursor: crosshair;
  color: var(--fg);
  touch-action: none;
  user-select: none;
}

.penstock {
  stroke: var(--accent);
  stroke-width: 2.5;
  stroke-linecap: round;
  stroke-linejoin: round;
  opacity: 0.85;
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
  stroke-width: 0.5;
}

.mark {
  fill: var(--panel);
  stroke: var(--accent);
  stroke-width: 0.35;
  vector-effect: non-scaling-stroke;
}

.mark-sq {
  fill: var(--panel);
  stroke: #c47b2b;
  stroke-width: 0.35;
  vector-effect: non-scaling-stroke;
}

.node.intake .mark {
  stroke: #2a7a4b;
  fill: color-mix(in srgb, #2a7a4b 25%, var(--panel));
}

.node.turbine .mark-sq {
  fill: color-mix(in srgb, #c47b2b 25%, var(--panel));
}

.node.selected .mark,
.node.selected .mark-sq {
  stroke-width: 0.55;
  filter: drop-shadow(0 0 0.4px var(--accent));
}

.node.selected .hit {
  stroke: var(--accent);
  stroke-opacity: 0.35;
  stroke-width: 0.8;
}

.label {
  font-size: 2.4px;
  fill: var(--muted-fg);
  pointer-events: none;
  user-select: none;
}

.empty-overlay {
  position: absolute;
  inset: 2.5rem 2rem 2rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: 0.45rem;
  padding: 1.25rem;
  border: 1px dashed var(--border-strong);
  border-radius: 12px;
  background: color-mix(in srgb, var(--panel) 88%, transparent);
  max-width: 26rem;
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
  max-width: 22rem;
  line-height: 1.45;
  font-size: 0.9rem;
  color: var(--fg);
}

.muted {
  color: var(--muted-fg) !important;
  font-size: 0.78rem !important;
}

.axis-hint {
  position: absolute;
  font-size: 0.65rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--muted-fg);
  pointer-events: none;
}

.axis-hint.y {
  left: 0.4rem;
  top: 50%;
  transform: translateY(-50%) rotate(-90deg);
  transform-origin: left center;
}

.axis-hint.x {
  bottom: 0.35rem;
  left: 50%;
  transform: translateX(-50%);
}

.status {
  margin: 0;
  padding: 0.45rem 0.75rem;
  font-size: 0.8rem;
  color: var(--muted-fg);
  border-top: 1px solid var(--border);
  background: var(--toolbar);
}
</style>
