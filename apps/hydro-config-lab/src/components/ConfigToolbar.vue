<script setup lang="ts">
import { ref, watch } from "vue";
import {
  deleteConfig,
  listConfigs,
  loadConfig,
  saveConfig,
  type ConfigIndexEntry,
} from "../lib/configStore";
import {
  createLabDocument,
  downloadText,
  exportPlantJson,
  parseImport,
} from "../lib/labDocument";
import type { OperatorInputs, PlantParams } from "../lib/plantParams";
import type { Site } from "../lib/site";

const props = defineProps<{
  name: string;
  site: Site;
  params: PlantParams;
  operator: OperatorInputs;
}>();

const emit = defineEmits<{
  "update:name": [string];
  load: [
    {
      name: string;
      site: Site;
      params: PlantParams;
      operator: OperatorInputs;
    },
  ];
  new: [];
}>();

const saved = ref<ConfigIndexEntry[]>([]);
const fileInput = ref<HTMLInputElement | null>(null);
const message = ref("");

function refreshList() {
  saved.value = listConfigs();
}

watch(
  () => props.name,
  () => refreshList(),
  { immediate: true },
);

function onSave() {
  const name = props.name.trim() || "Untitled site";
  saveConfig(name, props.site, props.params, props.operator);
  emit("update:name", name);
  refreshList();
  message.value = `Saved “${name}”`;
}

function onSaveAs() {
  const name = window.prompt("Save configuration as:", props.name);
  if (!name?.trim()) return;
  const key = name.trim();
  saveConfig(key, props.site, props.params, props.operator);
  emit("update:name", key);
  refreshList();
  message.value = `Saved “${key}”`;
}

function onLoad(name: string) {
  const doc = loadConfig(name);
  if (!doc) {
    message.value = `Could not load “${name}”`;
    return;
  }
  emit("load", {
    name: doc.name,
    site: doc.site,
    params: doc.params,
    operator: doc.operator,
  });
  message.value = `Loaded “${doc.name}”`;
}

function onDelete(name: string) {
  if (!confirm(`Delete saved config “${name}”?`)) return;
  deleteConfig(name);
  refreshList();
  message.value = `Deleted “${name}”`;
}

function onExportPlant() {
  const r = exportPlantJson(props.site, props.params);
  if (!r.ok) {
    message.value = r.error;
    return;
  }
  const id = r.plant.id || "plant";
  downloadText(`${id}.json`, r.text);
  message.value = `Exported ${id}.json — copy into fixtures/plants/ if desired`;
}

function onExportLab() {
  const doc = createLabDocument(props.name, props.site, props.params, props.operator);
  const slug = (props.name || "lab-site").replace(/[^\w.-]+/g, "-");
  downloadText(`${slug}.lab.json`, JSON.stringify(doc, null, 2) + "\n");
  message.value = "Exported lab document (includes site geometry)";
}

function onImportClick() {
  fileInput.value?.click();
}

async function onFile(ev: Event) {
  const input = ev.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (!file) return;
  try {
    const text = await file.text();
    const state = parseImport(text);
    emit("load", state);
    message.value = `Imported “${state.name}”`;
  } catch (e) {
    message.value = e instanceof Error ? e.message : String(e);
  }
}
</script>

<template>
  <div class="toolbar">
    <label class="name-field">
      <span class="sr">Configuration name</span>
      <input
        type="text"
        class="name-input"
        :value="name"
        placeholder="Untitled site"
        @change="emit('update:name', ($event.target as HTMLInputElement).value)"
      />
    </label>
    <button type="button" class="btn" @click="emit('new')">New</button>
    <button type="button" class="btn" @click="onSave">Save</button>
    <button type="button" class="btn" @click="onSaveAs">Save as…</button>
    <button type="button" class="btn" title="Engine hydro-plant JSON" @click="onExportPlant">
      Export plant
    </button>
    <button
      type="button"
      class="btn"
      title="Full lab document with site geometry"
      @click="onExportLab"
    >
      Export lab
    </button>
    <button type="button" class="btn" @click="onImportClick">Import…</button>
    <input
      ref="fileInput"
      type="file"
      accept="application/json,.json"
      class="hidden"
      @change="onFile"
    />

    <div v-if="saved.length" class="saved">
      <span class="label">Open saved:</span>
      <span v-for="c in saved" :key="c.name" class="chip-row">
        <button
          type="button"
          class="chip"
          :title="`Saved ${c.savedAt}`"
          @click="onLoad(c.name)"
        >
          {{ c.name }}
        </button>
        <button
          type="button"
          class="chip danger"
          :title="`Delete ${c.name}`"
          @click="onDelete(c.name)"
        >
          ×
        </button>
      </span>
    </div>
    <p v-if="message" class="msg">{{ message }}</p>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.45rem;
}

.name-field {
  margin-right: 0.15rem;
}

.name-input {
  font: inherit;
  font-size: 0.85rem;
  min-width: 10rem;
  padding: 0.35rem 0.55rem;
  border: 1px dashed var(--border);
  border-radius: 6px;
  background: var(--panel);
  color: var(--fg);
}

.btn {
  font: inherit;
  font-size: 0.85rem;
  padding: 0.4rem 0.75rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--panel);
  color: var(--fg);
  cursor: pointer;
}

.btn:hover {
  border-color: var(--accent);
}

.hidden {
  display: none;
}

.sr {
  position: absolute;
  width: 1px;
  height: 1px;
  overflow: hidden;
  clip: rect(0 0 0 0);
}

.saved {
  flex-basis: 100%;
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  align-items: center;
  margin-top: 0.15rem;
}

.label {
  font-size: 0.75rem;
  color: var(--muted-fg);
}

.chip-row {
  display: inline-flex;
  align-items: center;
  gap: 0.15rem;
}

.chip {
  font: inherit;
  font-size: 0.75rem;
  padding: 0.2rem 0.5rem;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--fg);
  cursor: pointer;
}

.chip:hover {
  border-color: var(--accent);
}

.chip.danger {
  color: #c44;
  padding: 0.2rem 0.4rem;
  opacity: 0.8;
}

.msg {
  flex-basis: 100%;
  margin: 0.15rem 0 0;
  font-size: 0.78rem;
  color: var(--muted-fg);
}
</style>
