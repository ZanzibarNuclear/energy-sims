<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
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

const open = ref(false);
const saved = ref<ConfigIndexEntry[]>([]);
const fileInput = ref<HTMLInputElement | null>(null);
const message = ref("");
const root = ref<HTMLElement | null>(null);

function refreshList() {
  saved.value = listConfigs();
}

watch(
  () => props.name,
  () => refreshList(),
  { immediate: true },
);

function close() {
  open.value = false;
}

function toggle() {
  open.value = !open.value;
  if (open.value) {
    message.value = "";
    refreshList();
  }
}

function onDocClick(ev: MouseEvent) {
  if (!open.value || !root.value) return;
  if (!root.value.contains(ev.target as Node)) close();
}

function onKey(ev: KeyboardEvent) {
  if (ev.key === "Escape") close();
}

onMounted(() => {
  document.addEventListener("click", onDocClick);
  document.addEventListener("keydown", onKey);
});
onUnmounted(() => {
  document.removeEventListener("click", onDocClick);
  document.removeEventListener("keydown", onKey);
});

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
  close();
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
  downloadText(`${r.plant.id || "plant"}.json`, r.text);
  message.value = "Exported plant JSON";
}

function onExportLab() {
  const doc = createLabDocument(props.name, props.site, props.params, props.operator);
  const slug = (props.name || "lab-site").replace(/[^\w.-]+/g, "-");
  downloadText(`${slug}.lab.json`, JSON.stringify(doc, null, 2) + "\n");
  message.value = "Exported lab document";
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
    const state = parseImport(await file.text());
    emit("load", state);
    message.value = `Imported “${state.name}”`;
    close();
  } catch (e) {
    message.value = e instanceof Error ? e.message : String(e);
  }
}

function onNew() {
  emit("new");
  close();
}
</script>

<template>
  <div ref="root" class="menu-wrap">
    <button
      type="button"
      class="burger"
      :aria-expanded="open"
      aria-haspopup="true"
      aria-label="File menu"
      @click.stop="toggle"
    >
      <span class="bar" />
      <span class="bar" />
      <span class="bar" />
    </button>

    <div v-if="open" class="dropdown" role="menu" @click.stop>
      <div class="name-row">
        <label>
          <span class="sr">Name</span>
          <input
            type="text"
            :value="name"
            placeholder="Untitled site"
            @change="emit('update:name', ($event.target as HTMLInputElement).value)"
          />
        </label>
      </div>
      <button type="button" role="menuitem" @click="onNew">New site…</button>
      <button type="button" role="menuitem" @click="onSave">Save</button>
      <button type="button" role="menuitem" @click="onSaveAs">Save as…</button>
      <hr />
      <button type="button" role="menuitem" @click="onExportPlant">Export plant JSON…</button>
      <button type="button" role="menuitem" @click="onExportLab">Export lab document…</button>
      <button type="button" role="menuitem" @click="onImportClick">Import…</button>
      <input
        ref="fileInput"
        type="file"
        accept="application/json,.json"
        class="hidden"
        @change="onFile"
      />
      <template v-if="saved.length">
        <hr />
        <p class="section">Open saved</p>
        <div v-for="c in saved" :key="c.name" class="saved-row">
          <button type="button" class="load" @click="onLoad(c.name)">{{ c.name }}</button>
          <button type="button" class="del" :title="`Delete ${c.name}`" @click="onDelete(c.name)">
            ×
          </button>
        </div>
      </template>
      <p v-if="message" class="msg">{{ message }}</p>
    </div>
  </div>
</template>

<style scoped>
.menu-wrap {
  position: relative;
}

.burger {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 4px;
  width: 2.4rem;
  height: 2.4rem;
  padding: 0.45rem;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--panel);
  cursor: pointer;
}

.burger:hover {
  border-color: var(--accent);
}

.bar {
  display: block;
  height: 2px;
  width: 100%;
  background: var(--fg);
  border-radius: 1px;
}

.dropdown {
  position: absolute;
  right: 0;
  top: calc(100% + 0.35rem);
  min-width: 15rem;
  max-width: min(20rem, 92vw);
  padding: 0.45rem;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--panel);
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.12);
  z-index: 40;
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.name-row input {
  width: 100%;
  font: inherit;
  font-size: 0.85rem;
  padding: 0.4rem 0.5rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--fg);
  margin-bottom: 0.25rem;
}

.dropdown button[role="menuitem"],
.dropdown .load {
  font: inherit;
  font-size: 0.88rem;
  text-align: left;
  padding: 0.45rem 0.55rem;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--fg);
  cursor: pointer;
}

.dropdown button[role="menuitem"]:hover,
.dropdown .load:hover {
  background: var(--bg);
}

.dropdown hr {
  border: none;
  border-top: 1px solid var(--border);
  margin: 0.3rem 0;
}

.section {
  margin: 0.15rem 0.4rem 0.1rem;
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--muted-fg);
}

.saved-row {
  display: flex;
  align-items: center;
  gap: 0.2rem;
}

.saved-row .load {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.del {
  font: inherit;
  border: none;
  background: transparent;
  color: #c44;
  cursor: pointer;
  padding: 0.3rem 0.45rem;
  border-radius: 4px;
}

.del:hover {
  background: color-mix(in srgb, #c44 12%, transparent);
}

.msg {
  margin: 0.35rem 0.4rem 0.15rem;
  font-size: 0.75rem;
  color: var(--muted-fg);
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
</style>
