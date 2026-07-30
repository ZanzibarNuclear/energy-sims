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

function updateHorizontal(raw: string) {
  if (!props.selection || !point.value) return;
  const n = Number(raw);
  if (!Number.isFinite(n)) return;
  emit(
    "update:site",
    setSelectedPoint(props.site, props.selection, {
      ...point.value,
      sM: n,
    }),
  );
}

function updateElevationRelative(raw: string) {
  if (!props.selection || !point.value) return;
  const n = Number(raw);
  if (!Number.isFinite(n)) return;
  // Display elev = z - origin → z = elev + origin
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
</script>

<template>
  <div class="selection">
    <h2>Selection</h2>
    <template v-if="selection && point">
      <p class="kind">{{ label }}</p>
      <label class="field">
        <span>Horizontal distance (m)</span>
        <input
          type="number"
          :step="GRID_STEP_M"
          :value="roundCoord(point.sM)"
          @change="updateHorizontal(($event.target as HTMLInputElement).value)"
        />
      </label>
      <label class="field">
        <span>
          {{
            hasTurbineDatum ? "Elevation above turbine (m)" : "Elevation (m)"
          }}
        </span>
        <input
          type="number"
          :step="GRID_STEP_M"
          :value="roundCoord(elevRelative)"
          @change="updateElevationRelative(($event.target as HTMLInputElement).value)"
        />
      </label>
      <p v-if="hasTurbineDatum" class="hint">
        Turbine is elevation 0. Pipe length is along the penstock (not the horizontal value).
      </p>
      <button type="button" class="danger" @click="onDelete">Delete element</button>
    </template>
    <p v-else class="placeholder">
      Nothing selected. Use a tool to place an intake, bends, or turbine — or click an
      existing point.
    </p>
  </div>
</template>

<style scoped>
.selection h2 {
  margin: 0 0 0.5rem;
  font-size: 0.95rem;
  font-weight: 650;
}

.kind {
  margin: 0 0 0.65rem;
  font-weight: 600;
  font-size: 0.9rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  margin-bottom: 0.55rem;
  font-size: 0.78rem;
  color: var(--muted-fg);
}

.field input {
  font: inherit;
  font-size: 0.9rem;
  color: var(--fg);
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0.35rem 0.5rem;
}

.hint {
  margin: 0 0 0.5rem;
  font-size: 0.75rem;
  line-height: 1.4;
  color: var(--muted-fg);
}

.danger {
  font: inherit;
  font-size: 0.8rem;
  margin-top: 0.35rem;
  padding: 0.35rem 0.6rem;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, #c44 50%, var(--border));
  background: transparent;
  color: #c44;
  cursor: pointer;
}

.danger:hover {
  background: color-mix(in srgb, #c44 12%, transparent);
}

.placeholder {
  margin: 0;
  font-size: 0.85rem;
  line-height: 1.45;
  color: var(--muted-fg);
}
</style>
