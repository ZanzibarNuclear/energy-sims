<script setup lang="ts">
import { computed } from "vue";
import {
  deleteSelection,
  getSelectedPoint,
  roundCoord,
  selectionLabel,
  setSelectedPoint,
  type Site,
  type SiteSelection,
} from "../lib/site";
import { elevationOriginZ, GRID_STEP_M } from "../lib/viewBox";

const props = defineProps<{
  site: Site;
  selection: SiteSelection;
}>();

const emit = defineEmits<{
  "update:site": [Site];
  "update:selection": [SiteSelection];
}>();

const label = computed(() => selectionLabel(props.selection));
const point = computed(() => getSelectedPoint(props.site, props.selection));
const elevOrigin = computed(() => elevationOriginZ(props.site.turbine?.zM));
const elevRelative = computed(() =>
  point.value ? point.value.zM - elevOrigin.value : 0,
);
const hasTurbineDatum = computed(() => props.site.turbine != null);
const visible = computed(() => !!(props.selection && point.value));

function updateHorizontal(raw: string) {
  if (!props.selection || !point.value) return;
  const n = Number(raw);
  if (!Number.isFinite(n)) return;
  emit(
    "update:site",
    setSelectedPoint(props.site, props.selection, { ...point.value, sM: n }),
  );
}

function updateElevationRelative(raw: string) {
  if (!props.selection || !point.value) return;
  const n = Number(raw);
  if (!Number.isFinite(n)) return;
  emit(
    "update:site",
    setSelectedPoint(props.site, props.selection, {
      ...point.value,
      zM: n + elevOrigin.value,
    }),
  );
}

function onDelete() {
  const r = deleteSelection(props.site, props.selection);
  emit("update:site", r.site);
  emit("update:selection", r.selection);
}

function onClose() {
  emit("update:selection", null);
}
</script>

<template>
  <div v-if="visible && point" class="popover" role="dialog" :aria-label="label">
    <header>
      <strong>{{ label }}</strong>
      <button type="button" class="close" aria-label="Close" @click="onClose">×</button>
    </header>
    <div class="fields">
      <label>
        <span>Horizontal (m)</span>
        <input
          type="number"
          :step="GRID_STEP_M"
          :value="roundCoord(point.sM)"
          @change="updateHorizontal(($event.target as HTMLInputElement).value)"
        />
      </label>
      <label>
        <span>{{ hasTurbineDatum ? "Elev. above turbine (m)" : "Elevation (m)" }}</span>
        <input
          type="number"
          :step="GRID_STEP_M"
          :value="roundCoord(elevRelative)"
          @change="updateElevationRelative(($event.target as HTMLInputElement).value)"
        />
      </label>
    </div>
    <button type="button" class="danger" @click="onDelete">Delete</button>
  </div>
</template>

<style scoped>
.popover {
  position: absolute;
  right: 0.75rem;
  top: 3.4rem;
  z-index: 5;
  width: min(16rem, calc(100% - 1.5rem));
  padding: 0.65rem 0.75rem 0.75rem;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--panel);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.1);
}

header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

header strong {
  font-size: 0.9rem;
}

.close {
  font: inherit;
  font-size: 1.2rem;
  line-height: 1;
  border: none;
  background: transparent;
  color: var(--muted-fg);
  cursor: pointer;
  padding: 0 0.2rem;
}

.fields {
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
}

label {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  font-size: 0.72rem;
  color: var(--muted-fg);
}

input {
  font: inherit;
  font-size: 0.88rem;
  color: var(--fg);
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.3rem 0.45rem;
}

.danger {
  margin-top: 0.55rem;
  font: inherit;
  font-size: 0.8rem;
  padding: 0.3rem 0.55rem;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, #c44 45%, var(--border));
  background: transparent;
  color: #c44;
  cursor: pointer;
}
</style>
