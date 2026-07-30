<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import AppMenu from "./components/AppMenu.vue";
import ServerStatus from "./components/ServerStatus.vue";
import SiteCanvas from "./components/SiteCanvas.vue";
import PlantForm from "./components/PlantForm.vue";
import SteadyPreview from "./components/SteadyPreview.vue";
import TrialRunner from "./components/TrialRunner.vue";
import { compileSite } from "./lib/compileSite";
import { clearDraft, loadDraft, saveDraft } from "./lib/draftStore";
import { emptyLabState } from "./lib/labDocument";
import type { OperatorInputs, PlantParams } from "./lib/plantParams";
import {
  isSiteComplete,
  normalizeSite,
  type Site,
  type SiteSelection,
} from "./lib/site";

type TabId = "layout" | "equipment" | "run";

const tabs: { id: TabId; step: string; label: string }[] = [
  { id: "layout", step: "1", label: "Layout" },
  { id: "equipment", step: "2", label: "Equipment" },
  { id: "run", step: "3", label: "Run" },
];

const instructions: Record<TabId, string> = {
  layout: "Lay out the penstock so that it runs downhill from intake to turbine.",
  equipment: "Tune stream, pipe, turbine, generator, and operator settings. Head and length stay from Layout.",
  run: "Start the energy-sim engine and play a timed run to watch power and speed ramp.",
};

const tab = ref<TabId>("layout");

const initial = emptyLabState();
const configName = ref(initial.name);
const site = ref<Site>(initial.site);
const selection = ref<SiteSelection>(null);
const params = ref<PlantParams>(initial.params);
const operator = ref<OperatorInputs>(initial.operator);
const ready = ref(false);

const compiled = computed(() => compileSite(site.value, params.value));
const plant = computed(() => (compiled.value.ok ? compiled.value.plant : null));
const derived = computed(() => (compiled.value.ok ? compiled.value.derived : null));
const layoutReady = computed(() => isSiteComplete(site.value));

onMounted(() => {
  const draft = loadDraft();
  if (draft) {
    configName.value = draft.name;
    site.value = normalizeSite(draft.site);
    params.value = draft.params;
    operator.value = draft.operator;
  }
  ready.value = true;
});

// Auto-save working state to the browser (resume after refresh).
watch(
  [configName, site, params, operator],
  () => {
    if (!ready.value) return;
    saveDraft(configName.value, site.value, params.value, operator.value);
  },
  { deep: true },
);

function newSite() {
  if (!confirm("Reset to the default layout (intake + turbine)?")) return;
  const s = emptyLabState();
  configName.value = s.name;
  site.value = s.site;
  selection.value = null;
  params.value = s.params;
  operator.value = s.operator;
  tab.value = "layout";
  clearDraft();
  saveDraft(configName.value, site.value, params.value, operator.value);
}

function onLoad(state: {
  name: string;
  site: Site;
  params: PlantParams;
  operator: OperatorInputs;
}) {
  configName.value = state.name;
  site.value = normalizeSite(state.site);
  params.value = state.params;
  operator.value = state.operator;
  selection.value = null;
}

function goTab(id: TabId) {
  tab.value = id;
  if (id !== "layout") selection.value = null;
}
</script>

<template>
  <div class="app">
    <header class="top">
      <div class="brand">
        <h1>Hydro Config Lab</h1>
        <p class="tagline">{{ configName }}</p>
      </div>
      <AppMenu
        :name="configName"
        :site="site"
        :params="params"
        :operator="operator"
        @update:name="configName = $event"
        @new="newSite"
        @load="onLoad"
      />
    </header>

    <nav class="tabs" role="tablist" aria-label="Workflow">
      <button
        v-for="t in tabs"
        :key="t.id"
        type="button"
        role="tab"
        class="tab"
        :class="{ active: tab === t.id }"
        :aria-selected="tab === t.id"
        @click="goTab(t.id)"
      >
        <span class="step">{{ t.step }}</span>
        <span class="lab">{{ t.label }}</span>
      </button>
    </nav>

    <p class="instruction" role="status">{{ instructions[tab] }}</p>

    <section
      v-show="tab === 'layout'"
      class="panel layout-panel"
      role="tabpanel"
      aria-label="Layout"
    >
      <SiteCanvas v-model:site="site" v-model:selection="selection" />
    </section>

    <section
      v-show="tab === 'equipment'"
      class="panel equipment-panel"
      role="tabpanel"
      aria-label="Equipment"
    >
      <div class="equipment-grid">
        <PlantForm
          :params="params"
          :operator="operator"
          @update:params="params = $event"
          @update:operator="operator = $event"
        />
        <div class="preview-col">
          <SteadyPreview
            :plant="plant"
            :operator="operator"
            :enabled="layoutReady"
            :derived="derived"
          />
          <p v-if="compiled.ok === false && layoutReady" class="compile-err">
            {{ compiled.error }}
          </p>
        </div>
      </div>
    </section>

    <section
      v-show="tab === 'run'"
      class="panel run-panel"
      role="tabpanel"
      aria-label="Run"
    >
      <ServerStatus />
      <TrialRunner
        :plant="plant"
        :operator="operator"
        :enabled="layoutReady"
      />
    </section>
  </div>
</template>

<style scoped>
.app {
  max-width: 72rem;
  margin: 0 auto;
  padding: 1rem 1.25rem 2rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  min-height: 100vh;
}

.top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
}

.brand h1 {
  margin: 0;
  font-size: 1.35rem;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.tagline {
  margin: 0.2rem 0 0;
  color: var(--muted-fg);
  font-size: 0.88rem;
}

.tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  padding: 0.25rem;
  border-radius: 10px;
  background: var(--toolbar);
  border: 1px solid var(--border);
}

.tab {
  flex: 1 1 6rem;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.45rem;
  font: inherit;
  font-size: 0.9rem;
  padding: 0.55rem 0.75rem;
  border: 1px solid transparent;
  border-radius: 8px;
  background: transparent;
  color: var(--muted-fg);
  cursor: pointer;
}

.tab:hover {
  color: var(--fg);
  background: color-mix(in srgb, var(--panel) 70%, transparent);
}

.tab.active {
  background: var(--panel);
  border-color: var(--border);
  color: var(--fg);
  font-weight: 600;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
}

.step {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.35rem;
  height: 1.35rem;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 700;
  background: var(--bg);
  color: var(--muted-fg);
}

.tab.active .step {
  background: var(--accent);
  color: #fff;
}

.instruction {
  margin: 0;
  padding: 0.85rem 1rem;
  font-size: 1.05rem;
  line-height: 1.45;
  font-weight: 500;
  color: var(--fg);
  background: color-mix(in srgb, var(--accent) 10%, var(--panel));
  border: 1px solid color-mix(in srgb, var(--accent) 28%, var(--border));
  border-radius: 10px;
}

.panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
  min-height: 0;
}

.layout-panel :deep(.canvas) {
  min-height: 30rem;
  height: 100%;
}

.layout-panel :deep(.plot) {
  min-height: 26rem;
}

.equipment-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(13rem, 16rem);
  gap: 1rem;
  align-items: start;
}

@media (max-width: 800px) {
  .equipment-grid {
    grid-template-columns: 1fr;
  }
}

.equipment-panel {
  /* no extra chrome — PlantForm + SteadyPreview carry their own cards */
  padding: 0;
}

.preview-col {
  position: sticky;
  top: 0.5rem;
}

.compile-err {
  margin: 0;
  font-size: 0.8rem;
  color: #c44;
}

.run-panel {
  gap: 0.85rem;
}
</style>
