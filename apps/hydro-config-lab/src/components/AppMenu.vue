<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import {
  createLabDocument,
  downloadLabFile,
  parseLabFile,
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
const fileInput = ref<HTMLInputElement | null>(null);
const message = ref("");
const root = ref<HTMLElement | null>(null);

function close() {
  open.value = false;
}

function toggle() {
  open.value = !open.value;
  if (open.value) message.value = "";
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

/** Save = download a full configuration JSON you can open in an editor. */
function onSave() {
  const name = props.name.trim() || "Untitled site";
  emit("update:name", name);
  const doc = createLabDocument(name, props.site, props.params, props.operator);
  const filename = downloadLabFile(doc);
  message.value = `Saved ${filename}`;
}

function onOpenClick() {
  fileInput.value?.click();
}

async function onFile(ev: Event) {
  const input = ev.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (!file) return;
  try {
    const state = parseLabFile(await file.text());
    emit("load", state);
    message.value = `Opened “${state.name}”`;
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
      <button type="button" role="menuitem" @click="onSave">Save…</button>
      <button type="button" role="menuitem" @click="onOpenClick">Open…</button>
      <input
        ref="fileInput"
        type="file"
        accept="application/json,.json"
        class="hidden"
        @change="onFile"
      />
      <p class="hint">
        Save downloads a JSON file (layout + equipment). Open loads it back. The browser also
        auto-saves a draft for refresh.
      </p>
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
  min-width: 16rem;
  max-width: min(22rem, 92vw);
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

.dropdown button[role="menuitem"] {
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

.dropdown button[role="menuitem"]:hover {
  background: var(--bg);
}

.hint {
  margin: 0.4rem 0.4rem 0.15rem;
  font-size: 0.72rem;
  line-height: 1.4;
  color: var(--muted-fg);
}

.msg {
  margin: 0.25rem 0.4rem 0.15rem;
  font-size: 0.75rem;
  color: var(--fg);
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
