<script setup lang="ts">
import { computed, ref } from "vue";
import AppMenu from "./components/AppMenu.vue";
import ServerStatus from "./components/ServerStatus.vue";
import SiteCanvas from "./components/SiteCanvas.vue";
import SelectionPopover from "./components/SelectionPopover.vue";
import PlantForm from "./components/PlantForm.vue";
import SteadyPreview from "./components/SteadyPreview.vue";
import TrialRunner from "./components/TrialRunner.vue";
import { compileSite } from "./lib/compileSite";
import { emptyLabState } from "./lib/labDocument";
import type { OperatorInputs, PlantParams } from "./lib/plantParams";
import {
  isSiteComplete,
  type Site,
  type SiteSelection,
  type ToolId,
} from "./lib/site";

type TabId = "layout" | "equipment" | "run";

const tabs: { id: TabId; step: string; label: string; hint: string }[] = [
  { id: "layout", step: "1", label: "Layout", hint: "Place intake, penstock, turbine" },
  { id: "equipment", step: "2", label: "Equipment", hint: "Flow, pipe, losses, efficiencies" },
  { id: "run", step: "3", label: "Run", hint: "Play simulation against the engine" },
];

const tab = ref<TabId>("layout");

const initial = emptyLabState();
const configName = ref(initial.name);
const site = ref<Site>(initial.site);
const selection = ref<SiteSelection>(null);
const tool = ref<ToolId>("intake");
const params = ref<PlantParams>(initial.params);
const operator = ref<OperatorInputs>(initial.operator);

const compiled = computed(() => compileSite(site.value, params.value));
const plant = computed(() => (compiled.value.ok ? compiled.value.plant : null));
const derived = computed(() => (compiled.value.ok ? compiled.value.derived : null));
const layoutReady = computed(() => isSiteComplete(site.value));

function newSite() {
  if (
    (site.value.intake || site.value.turbine || site.value.bends.length) &&
    !confirm("Discard the current site and start a clean slate?")
  ) {
    return;
  }
  const s = emptyLabState();
  configName.value = s.name;
  site.value = s.site;
  selection.value = null;
  tool.value = "intake";
  params.value = s.params;
  operator.value = s.operator;
  tab.value = "layout";
}

function onLoad(state: {
  name: string;
  site: Site;
  params: PlantParams;
  operator: OperatorInputs;
}) {
  configName.value = state.name;
  site.value = state.site;
  params.value = state.params;
  operator.value = state.operator;
  selection.value = null;
  tool.value = "select";
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
        :title="t.hint"
        @click="goTab(t.id)"
      >
        <span class="step">{{ t.step }}</span>
        <span class="lab">{{ t.label }}</span>
      </button>
    </nav>

    <!-- 1. Layout: grid only + selection popover -->
    <section
      v-show="tab === 'layout'"
      class="panel layout-panel"
      role="tabpanel"
      aria-label="Layout"
    >
      <div class="canvas-host">
        <SiteCanvas
          v-model:site="site"
          v-model:selection="selection"
          v-model:tool="tool"
        />
        <SelectionPopover v-model:site="site" v-model:selection="selection" />
      </div>
      <p class="step-hint">
        Place intake and turbine on the grid. Optional bends shape the penstock.
        <button
          v-if="layoutReady"
          type="button"
          class="link"
          @click="goTab('equipment')"
        >
          Next: Equipment →
        </button>
      </p>
    </section>

    <!-- 2. Equipment & stream (avoid "plant" in the tab chrome) -->
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
          :derived="derived"
          :complete="layoutReady"
          @update:params="params = $event"
          @update:operator="operator = $event"
        />
        <div class="preview-col">
          <SteadyPreview
            :plant="plant"
            :operator="operator"
            :enabled="layoutReady"
          />
          <p v-if="compiled.ok === false && layoutReady" class="compile-err">
            {{ compiled.error }}
          </p>
          <p v-if="layoutReady" class="step-hint">
            <button type="button" class="link" @click="goTab('run')">Next: Run →</button>
          </p>
          <p v-else class="step-hint warn">
            Layout is incomplete —
            <button type="button" class="link" @click="goTab('layout')">return to Layout</button>
          </p>
        </div>
      </div>
    </section>

    <!-- 3. Run against engine -->
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
  gap: 0.85rem;
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

.panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
  min-height: 0;
}

.layout-panel .canvas-host {
  position: relative;
  flex: 1;
  min-height: 28rem;
}

.layout-panel :deep(.canvas) {
  min-height: 28rem;
  height: 100%;
}

.layout-panel :deep(.plot) {
  min-height: 24rem;
}

.step-hint {
  margin: 0;
  font-size: 0.85rem;
  color: var(--muted-fg);
}

.step-hint.warn {
  color: #b8860b;
}

.link {
  font: inherit;
  font-size: inherit;
  color: var(--accent);
  background: none;
  border: none;
  cursor: pointer;
  text-decoration: underline;
  padding: 0;
}

.equipment-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.2fr) minmax(14rem, 18rem);
  gap: 1.25rem;
  align-items: start;
}

@media (max-width: 800px) {
  .equipment-grid {
    grid-template-columns: 1fr;
  }
}

.equipment-panel {
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--panel);
  padding: 1rem 1.1rem 1.25rem;
}

.preview-col {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  padding: 0.75rem;
  border-radius: 8px;
  background: var(--bg);
  border: 1px solid var(--border);
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
